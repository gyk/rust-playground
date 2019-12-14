void do_thing(int (*cb)(void*, int), void* context) {
    cb(context, 10);
}
