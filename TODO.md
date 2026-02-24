TODO
===

- [ ] independent backing store
  - [x] set up a backing sqlite db
  - [x] write test notes to sqlite db successfully
	- [ ] write test note directory
  - [x] ingest org-roam notes into our backing store
	- [x] test successful idempotent ingestion
	- [x] read org-roam notes
	- [x] serialize into our Note struct
  - [x] migration of MTIMES from org to our sqlite
- [ ] directory note source
- [ ] list-of-links note source
  - [ ] ingest list of links into store
- [ ] RSS feed
- [ ] working rust rewrite
- [x] UPSTREAM WORK SO FAR HOLY SHIT
- [ ] render sorted notes to RSS feed
- [ ] sort test notes
- [ ] render test notes
- [x] successful edge creation
- [ ] CLI frontend (subcommands: current, recent, review, visit, import)


## Rust learning avenues

- [ ] custom error types (replace `Box<dyn Error>` / `.unwrap()` with `thiserror` or manual impls)
- [ ] lifetimes (understand where cloning can be replaced with borrowing)
- [ ] trait design (e.g. shared `DirtyTracked` trait for `NoteStore` and `LinkGraph`)
- [ ] idiomatic method signatures (`&mut self` vs `self: &mut Self`)
- [ ] explore the type system more: generics, associated types, trait objects vs enums

## Features I'm thinking about:

- weighted links
- suggested indices/tags
- new note suggestions (or "suggested stubs")
  - what should be an index and what should be a new note? These migth be the same thing.
- ways to introspect on toricelli usage and analyse edit histories "with vs without toricelli"
- populating edit histories from before the MTIMES metadata by looking at note-wise commit histories. 
	- this gets complex in my org notebook, where a note isn't always a file but can often be a heading. (it always has the same ID.)
- "semantic histories," finding paths of keywords, citations, and semantic clusters through the commit histories
