use std::path::Path;
use std::sync::Arc;

use crate::Result;
use crate::db::db_pool::DbPool;
use crate::db::{
    app::AppRepo, oauth_code::OauthCodeRepo, org::OrgRepo, org_app::OrgAppRepo,
    org_member::OrgMemberRepo, password::PasswordRepo, superuser::SuperuserRepo, user::UserRepo,
};

pub struct DbMapper {
    pub apps: AppRepo,
    pub oauth_codes: OauthCodeRepo,
    pub orgs: OrgRepo,
    pub org_apps: OrgAppRepo,
    pub org_members: OrgMemberRepo,
    pub passwords: PasswordRepo,
    pub superusers: SuperuserRepo,
    pub users: UserRepo,
}

pub async fn create_db_mapper(filename: &Path, pool_size: usize) -> Result<DbMapper> {
    let pool = DbPool::new(filename, pool_size).await?;
    let arc_pool = Arc::new(pool);

    Ok(DbMapper {
        apps: AppRepo::new(arc_pool.clone()),
        oauth_codes: OauthCodeRepo::new(arc_pool.clone()),
        orgs: OrgRepo::new(arc_pool.clone()),
        org_apps: OrgAppRepo::new(arc_pool.clone()),
        org_members: OrgMemberRepo::new(arc_pool.clone()),
        passwords: PasswordRepo::new(arc_pool.clone()),
        superusers: SuperuserRepo::new(arc_pool.clone()),
        users: UserRepo::new(arc_pool.clone()),
    })
}
