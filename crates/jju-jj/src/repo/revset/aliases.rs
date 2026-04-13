use jj_lib::revset;

pub(super) fn aliases_map() -> revset::RevsetAliasesMap {
    let mut aliases_map = revset::RevsetAliasesMap::new();

    let default_aliases = [
        (
            "trunk()",
            r#"latest(
              remote_bookmarks(exact:"main", exact:"origin") |
              remote_bookmarks(exact:"master", exact:"origin") |
              remote_bookmarks(exact:"trunk", exact:"origin") |
              remote_bookmarks(exact:"main", exact:"upstream") |
              remote_bookmarks(exact:"master", exact:"upstream") |
              remote_bookmarks(exact:"trunk", exact:"upstream") |
              root()
            )"#,
        ),
        (
            "builtin_immutable_heads()",
            "trunk() | tags() | untracked_remote_bookmarks()",
        ),
        ("immutable_heads()", "builtin_immutable_heads()"),
        ("immutable()", "::(immutable_heads() | root())"),
        ("mutable()", "~immutable()"),
    ];

    for (name, definition) in default_aliases {
        let _ = aliases_map.insert(name, definition);
    }
    aliases_map
}
