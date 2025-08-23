struct BasicTypes {
    bool b;

    char ch;
    unsigned char uch;
    char16_t ch16;
    char32_t ch32;
    wchar_t wch;

    short s16;
    unsigned short u16;
    int s32;
    unsigned int u32;
    long ssize;
    unsigned long usize;
    long long s64;
    unsigned long long u64;

    float f32;
    double f64;
    long double ld;

    int &ref;
    void *ptr;
    void (*funcptr)();
    bool BasicTypes::*memptr;
    void (BasicTypes::*memfuncptr)();
    char arr[10];

    enum {
        A,
        B,
        C
    } e;
    struct {
        int x;
    } s;
    class {
        int y;
    } c;
    union {
        int z;
    } u;
};
