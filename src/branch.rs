use git2::{BranchType, Branches, Error, ErrorCode, Repository, build::CheckoutBuilder};

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

pub enum BranchQuery {
    Local,
    Remote,
    LocalAndRemote,
}

fn parse_branches(
    repo: &Repository,
    mut branches: Branches<'_>,
    branch_type: BranchType,
    items: &mut Vec<BranchItem>,
) {
    while let Some(Ok((branch, _))) = branches.next() {
        if let Ok(name_opt) = branch.name() {
            let mut name = name_opt.unwrap_or_default().to_string();

            if branch_type == BranchType::Remote {
                if let Some((remote, branch_name)) = name.split_once('/') {
                    name = format!("{remote}/{branch_name}");
                }
            }

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

pub fn query_branches(repo: &Repository, branch_query: &BranchQuery) -> Vec<BranchItem> {
    let mut items = Vec::new();

    match branch_query {
        BranchQuery::Local => {
            if let Ok(branches) = repo.branches(Some(BranchType::Local)) {
                parse_branches(repo, branches, BranchType::Local, &mut items);
            }
        }
        BranchQuery::Remote => {
            if let Ok(branches) = repo.branches(Some(BranchType::Remote)) {
                parse_branches(repo, branches, BranchType::Remote, &mut items);
            }
        }
        BranchQuery::LocalAndRemote => {
            if let Ok(branches) = repo.branches(Some(BranchType::Local)) {
                parse_branches(repo, branches, BranchType::Local, &mut items);
            }
            if let Ok(branches) = repo.branches(Some(BranchType::Remote)) {
                parse_branches(repo, branches, BranchType::Remote, &mut items);
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
