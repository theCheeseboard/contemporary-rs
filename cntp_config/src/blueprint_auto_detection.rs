use git2::{Error, Repository};
use std::path::Path;

pub fn autodetect_blueprint(path: &Path) -> Result<bool, Error> {
    let repository = Repository::discover(path)?;
    let head = repository.head()?.peel_to_commit()?;

    // Find any tags pointing to HEAD
    let mut tags = Vec::new();
    repository.tag_foreach(|oid, _| {
        tags.push(oid);
        true
    })?;

    for tag_oid in tags {
        let object = repository.find_object(tag_oid, None)?;
        let commit = object.peel_to_commit()?;
        if head.id() == commit.id() {
            return Ok(false);
        }
    }

    // Didn't find a tag pointing to HEAD so this is probably a blueprint build
    Ok(true)
}
