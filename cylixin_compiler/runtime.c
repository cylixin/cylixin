/*
 * Cylixin Runtime — lightweight C runtime for set and dictionary support.
 * Compile: gcc output.s runtime.c -o program -lm
 *
 * Both cy_set and cy_dict use open-addressing hash tables with linear probing.
 * Keys and values are stored as int64_t (same boxing scheme as arrays).
 */

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <errno.h>

/* ── Hash table entry ─────────────────────────────────────────────── */

typedef struct {
    int64_t key;
    int64_t value;
    int     occupied;   /* 0 = empty, 1 = occupied, 2 = tombstone */
} Entry;

/* ── cy_dict (hash map: int64 → int64) ───────────────────────────── */

typedef struct {
    Entry  *entries;
    int64_t capacity;
    int64_t count;
} CyDict;

static uint64_t hash_i64(int64_t key) {
    uint64_t h = (uint64_t)key;
    h ^= h >> 33;
    h *= 0xff51afd7ed558ccdULL;
    h ^= h >> 33;
    h *= 0xc4ceb9fe1a85ec53ULL;
    h ^= h >> 33;
    return h;
}

static void cy_dict_grow(CyDict *d);

CyDict *cy_dict_new(void) {
    CyDict *d = (CyDict *)malloc(sizeof(CyDict));
    d->capacity = 16;
    d->count    = 0;
    d->entries  = (Entry *)calloc((size_t)d->capacity, sizeof(Entry));
    return d;
}

void cy_dict_set(CyDict *d, int64_t key, int64_t value) {
    if (d->count * 2 >= d->capacity) cy_dict_grow(d);

    uint64_t idx = hash_i64(key) % (uint64_t)d->capacity;
    for (;;) {
        Entry *e = &d->entries[idx];
        if (e->occupied == 0 || e->occupied == 2) {
            e->key      = key;
            e->value    = value;
            e->occupied = 1;
            d->count++;
            return;
        }
        if (e->key == key) {
            e->value = value;   /* update existing */
            return;
        }
        idx = (idx + 1) % (uint64_t)d->capacity;
    }
}

int64_t cy_dict_get(CyDict *d, int64_t key) {
    uint64_t idx = hash_i64(key) % (uint64_t)d->capacity;
    for (;;) {
        Entry *e = &d->entries[idx];
        if (e->occupied == 0) return 0;         /* not found → 0 */
        if (e->occupied == 1 && e->key == key)
            return e->value;
        idx = (idx + 1) % (uint64_t)d->capacity;
    }
}

int64_t cy_dict_has(CyDict *d, int64_t key) {
    uint64_t idx = hash_i64(key) % (uint64_t)d->capacity;
    for (;;) {
        Entry *e = &d->entries[idx];
        if (e->occupied == 0) return 0;
        if (e->occupied == 1 && e->key == key) return 1;
        idx = (idx + 1) % (uint64_t)d->capacity;
    }
}

int64_t cy_dict_size(CyDict *d) {
    return d->count;
}

void cy_dict_remove(CyDict *d, int64_t key) {
    uint64_t idx = hash_i64(key) % (uint64_t)d->capacity;
    for (;;) {
        Entry *e = &d->entries[idx];
        if (e->occupied == 0) return;           /* not found */
        if (e->occupied == 1 && e->key == key) {
            e->occupied = 2;                    /* tombstone */
            d->count--;
            return;
        }
        idx = (idx + 1) % (uint64_t)d->capacity;
    }
}

static void cy_dict_grow(CyDict *d) {
    int64_t old_cap = d->capacity;
    Entry  *old     = d->entries;

    d->capacity *= 2;
    d->count     = 0;
    d->entries   = (Entry *)calloc((size_t)d->capacity, sizeof(Entry));

    for (int64_t i = 0; i < old_cap; i++) {
        if (old[i].occupied == 1)
            cy_dict_set(d, old[i].key, old[i].value);
    }
    free(old);
}

/* ── cy_set (backed by cy_dict with dummy value = 1) ─────────────── */

typedef CyDict CySet;

CySet *cy_set_new(void) {
    return cy_dict_new();
}

void cy_set_add(CySet *s, int64_t key) {
    cy_dict_set(s, key, 1);
}

int64_t cy_set_contains(CySet *s, int64_t key) {
    return cy_dict_has(s, key);
}

void cy_set_remove(CySet *s, int64_t key) {
    cy_dict_remove(s, key);
}

int64_t cy_set_size(CySet *s) {
    return cy_dict_size(s);
}

/* ── cy_read — type-directed input from stdin ────────────────────── */

#define CY_READ_BUF 256

int64_t cy_read_int(const char *prompt) {
    char buf[CY_READ_BUF];
    while (1) {
        printf("%s", prompt);
        fflush(stdout);
        if (fgets(buf, CY_READ_BUF, stdin) == NULL) {
            fprintf(stderr, "\nError: unexpected end of input while reading int\n");
            exit(1);
        }
        buf[strcspn(buf, "\n")] = '\0';
        char *end;
        errno = 0;
        long long val = strtoll(buf, &end, 10);
        if (end != buf && *end == '\0' && errno == 0) {
            return (int64_t)val;
        }
        fprintf(stderr, "  \xe2\x9c\x97 Expected an integer, got '%s'. Try again.\n", buf);
    }
}

double cy_read_float(const char *prompt) {
    char buf[CY_READ_BUF];
    while (1) {
        printf("%s", prompt);
        fflush(stdout);
        if (fgets(buf, CY_READ_BUF, stdin) == NULL) {
            fprintf(stderr, "\nError: unexpected end of input while reading float\n");
            exit(1);
        }
        buf[strcspn(buf, "\n")] = '\0';
        char *end;
        errno = 0;
        double val = strtod(buf, &end);
        if (end != buf && *end == '\0' && errno == 0) {
            return val;
        }
        fprintf(stderr, "  \xe2\x9c\x97 Expected a number, got '%s'. Try again.\n", buf);
    }
}

char *cy_read_str(const char *prompt) {
    char buf[CY_READ_BUF];
    printf("%s", prompt);
    fflush(stdout);
    if (fgets(buf, CY_READ_BUF, stdin) == NULL) {
        fprintf(stderr, "\nError: unexpected end of input while reading string\n");
        exit(1);
    }
    buf[strcspn(buf, "\n")] = '\0';
    char *result = (char *)malloc(strlen(buf) + 1);
    strcpy(result, buf);
    return result;
}

int64_t cy_read_bool(const char *prompt) {
    char buf[CY_READ_BUF];
    while (1) {
        printf("%s", prompt);
        fflush(stdout);
        if (fgets(buf, CY_READ_BUF, stdin) == NULL) {
            fprintf(stderr, "\nError: unexpected end of input while reading bool\n");
            exit(1);
        }
        buf[strcspn(buf, "\n")] = '\0';
        if (strcmp(buf, "true") == 0 || strcmp(buf, "1") == 0)  return 1;
        if (strcmp(buf, "false") == 0 || strcmp(buf, "0") == 0) return 0;
        fprintf(stderr, "  \xe2\x9c\x97 Expected true or false, got '%s'. Try again.\n", buf);
    }
}

int8_t cy_read_char(const char *prompt) {
    char buf[CY_READ_BUF];
    while (1) {
        printf("%s", prompt);
        fflush(stdout);
        if (fgets(buf, CY_READ_BUF, stdin) == NULL) {
            fprintf(stderr, "\nError: unexpected end of input while reading char\n");
            exit(1);
        }
        buf[strcspn(buf, "\n")] = '\0';
        if (strlen(buf) == 1) {
            return (int8_t)buf[0];
        }
        fprintf(stderr, "  \xe2\x9c\x97 Expected a single character, got '%s'. Try again.\n", buf);
    }
}
