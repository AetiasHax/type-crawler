typedef enum {
    Flag1 = 1 << 0,
    Flag2 = 1 << 1,
    Flag3 = 1 << 2,
} Flags;

#define GROUP_SIZE 100

typedef enum {
    Thing1     = GROUP_SIZE,
    Thing2     = GROUP_SIZE * 2,
    Thing3     = GROUP_SIZE * 3,
    Thing3a    = Thing3 + 1,
    WeirdThing = (Thing1 * 3 + Thing2 * 7 + Thing3a * 5) / 5,
} Thing;
