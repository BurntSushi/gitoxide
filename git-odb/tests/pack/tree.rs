#[test]
fn using_option_as_data_does_not_increase_size_in_memory() {
    enum Cache {
        Unset,
        Decompressed(Vec<u8>),
        /// compressed bytes + decompressed size
        Compressed(Vec<u8>, usize),
    }
    enum ObjectKind {
        Base(git_object::Kind),
        OfsDelta(u64),
    }
    struct Entry {
        pub _pack_offset: u64,
        pub _entry_len: usize,
        pub _kind: ObjectKind,
        pub _crc32: u32,
        pub _cache: Cache,
    }

    struct TreeItem<D> {
        _offset: u64,
        _data: D,
        _children: Vec<usize>,
    }
    struct TreeItemOption<D> {
        _offset: u64,
        _data: Option<D>,
        _children: Vec<usize>,
    }
    assert_eq!(
        std::mem::size_of::<TreeItem<Entry>>(),
        std::mem::size_of::<TreeItemOption<Entry>>(),
        "we hope niche filling optimizations kick in for our data structures to not pay for the Option at all"
    );
}
