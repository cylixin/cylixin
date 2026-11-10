// Handles issue comment commands:
//   /assign-me          assign yourself to the issue (anyone)
//   /assign @user       assign someone else (maintainers with triage access or above)
//
// Run by .github/workflows/assign-command.yml through actions/github-script.
// The comment text is untrusted input: it is only matched with regexes here and
// never passed to a shell.

const MAINTAINER_PERMISSIONS = ['admin', 'maintain', 'write', 'triage'];
const TODO_LABEL = 'status: to do';
const IN_PROGRESS_LABEL = 'status: in progress';

function parseCommand(body) {
  // Commands must start a line, so quoting or mentioning them mid-sentence does nothing.
  if (/^\/assign-me\s*$/m.test(body)) return { kind: 'self' };
  const m = body.match(/^\/assign\s+@?([A-Za-z0-9](?:[A-Za-z0-9]|-(?=[A-Za-z0-9])){0,38})\s*$/m);
  if (m) return { kind: 'other', user: m[1] };
  return null;
}

module.exports = async ({ github, context, core }) => {
  const { owner, repo } = context.repo;
  const issue = context.payload.issue;
  const comment = context.payload.comment;
  const commenter = comment.user.login;

  const command = parseCommand(comment.body || '');
  if (!command) {
    core.info('No assign command in this comment.');
    return;
  }

  const reply = (body) =>
    github.rest.issues.createComment({ owner, repo, issue_number: issue.number, body });
  const react = (content) =>
    github.rest.reactions.createForIssueComment({ owner, repo, comment_id: comment.id, content });

  if (issue.pull_request) {
    await reply(`@${commenter} assign commands only work on issues, not pull requests.`);
    return;
  }
  if (issue.state !== 'open') {
    await reply(`@${commenter} this issue is closed, so it can't be assigned.`);
    return;
  }

  let target = commenter;
  if (command.kind === 'other') {
    // role_name is the real role (triage, maintain, ...); permission is the legacy
    // admin/write/read/none value, used as a fallback.
    let allowed = false;
    try {
      const { data: perm } = await github.rest.repos.getCollaboratorPermissionLevel({
        owner, repo, username: commenter,
      });
      allowed = MAINTAINER_PERMISSIONS.includes(perm.role_name) ||
        ['admin', 'write'].includes(perm.permission);
    } catch (err) {
      if (err.status !== 404) throw err;
    }
    if (!allowed) {
      await reply(
        `@${commenter} only maintainers can assign other people. ` +
        'To take this issue yourself, comment `/assign-me`.'
      );
      return;
    }
    target = command.user;
  }

  const current = (issue.assignees || []).map((a) => a.login.toLowerCase());
  if (current.includes(target.toLowerCase())) {
    await reply(`@${target} is already assigned to this issue.`);
    return;
  }
  // Self-assignment never takes an issue away from someone already working on it.
  if (command.kind === 'self' && current.length > 0) {
    const others = issue.assignees.map((a) => `@${a.login}`).join(', ');
    await reply(
      `@${commenter} this issue is already assigned to ${others}. ` +
      'Ask in this thread if you would like to help, or pick another ' +
      `[good first issue](https://github.com/${owner}/${repo}/labels/good%20first%20issue).`
    );
    return;
  }

  // GitHub only lets you assign collaborators and people who have commented on the issue.
  try {
    await github.rest.issues.checkUserCanBeAssigned({ owner, repo, assignee: target });
  } catch (err) {
    if (err.status !== 404) throw err;
    const commented = await github.paginate(github.rest.issues.listComments, {
      owner, repo, issue_number: issue.number, per_page: 100,
    });
    if (!commented.some((c) => c.user.login.toLowerCase() === target.toLowerCase())) {
      await reply(
        `@${commenter} GitHub can't assign @${target} yet. ` +
        `Ask @${target} to leave a comment on this issue first, then run the command again.`
      );
      return;
    }
  }

  const { data: updated } = await github.rest.issues.addAssignees({
    owner, repo, issue_number: issue.number, assignees: [target],
  });
  // addAssignees silently ignores users it can't assign, so check the result.
  if (!updated.assignees.some((a) => a.login.toLowerCase() === target.toLowerCase())) {
    await reply(`@${commenter} GitHub didn't accept @${target} as an assignee. Check the username and try again.`);
    return;
  }

  const labels = issue.labels.map((l) => (typeof l === 'string' ? l : l.name));
  if (labels.includes(TODO_LABEL)) {
    await github.rest.issues.removeLabel({ owner, repo, issue_number: issue.number, name: TODO_LABEL })
      .catch((err) => core.warning(`Could not remove '${TODO_LABEL}': ${err.message}`));
  }
  if (!labels.includes(IN_PROGRESS_LABEL)) {
    await github.rest.issues.addLabels({ owner, repo, issue_number: issue.number, labels: [IN_PROGRESS_LABEL] })
      .catch((err) => core.warning(`Could not add '${IN_PROGRESS_LABEL}': ${err.message}`));
  }

  await react('+1');
  await reply(
    command.kind === 'self'
      ? `Thanks @${target}, this issue is now assigned to you! 🎉 See [CONTRIBUTING.md](https://github.com/${owner}/${repo}/blob/main/CONTRIBUTING.md) to get started, and feel free to ask questions here.`
      : `Assigned @${target} to this issue (requested by @${commenter}).`
  );
};

module.exports.parseCommand = parseCommand;
