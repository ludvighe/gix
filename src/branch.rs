use git2::{BranchType, Error, ErrorCode, Repository, build::CheckoutBuilder};

pub struct BranchItem {
    pub name: String,
    pub oid: String,
    pub summary: String,
    pub is_head: bool,
    pub has_upstream: bool,
    pub is_gone: bool,
}

impl BranchItem {
    pub fn short_oid(&self) -> String {
        self.oid.chars().take(7).collect()
    }
}

pub fn get_branches(repo: &Repository) -> Vec<BranchItem> {
    let mut items = Vec::new();
    if let Ok(mut it) = repo.branches(Some(BranchType::Local)) {
        while let Some(Ok((branch, _))) = it.next() {
            if let Ok(name_opt) = branch.name() {
                let name = name_opt.unwrap_or_default().to_string();
                let commit = branch.get().peel_to_commit().ok();
                let oid_full = commit
                    .as_ref()
                    .map(|c| c.id().to_string())
                    .unwrap_or_default();
                let summary = commit
                    .as_ref()
                    .and_then(|c| c.summary().map(|s| s.to_string()))
                    .unwrap_or_default();

                let cfg = repo.config().ok();
                let remote_key = format!("branch.{}.remote", name);
                let merge_key = format!("branch.{}.merge", name);
                let has_cfg = cfg
                    .as_ref()
                    .map(|c| c.get_string(&remote_key).is_ok() && c.get_string(&merge_key).is_ok())
                    .unwrap_or(false);

                let upstream_res = branch.upstream();
                let has_upstream = upstream_res.is_ok();
                let is_gone = has_cfg
                    && matches!(
                        upstream_res.err().map(|e| e.code()),
                        Some(ErrorCode::NotFound)
                    );

                items.push(BranchItem {
                    name,
                    oid: oid_full,
                    summary,
                    is_head: branch.is_head(),
                    has_upstream,
                    is_gone,
                });
            }
        }
    }
    items
}

pub fn checkout_branch(repo: &Repository, name: &str) -> Result<(), Error> {
    let mut cb = CheckoutBuilder::new();
    cb.safe();

    let branch = repo.find_branch(name, BranchType::Local)?;
    let reference = branch.get();
    let commit = reference.peel_to_commit()?;
    repo.checkout_tree(commit.as_object(), Some(&mut cb))?;
    repo.set_head(
        reference
            .name()
            .ok_or_else(|| Error::from_str("invalid ref name"))?,
    )?;
    Ok(())
}
