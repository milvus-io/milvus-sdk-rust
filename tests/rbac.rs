use milvus::client::ClientBuilder;
use milvus::error::Result;

mod common;
use common::*;

#[tokio::test]
async fn rbac_user_role_privilege_and_group_lifecycle() -> Result<()> {
    let client = ClientBuilder::new(URL)
        .username("root")
        .password("Milvus")
        .build()
        .await?;
    let suffix = gen_random_name();
    let user = format!("test_user_{}", suffix);
    let role = format!("test_role_{}", suffix);
    let group = format!("test_group_{}", suffix);
    let password = "MilvusTest123!";
    let new_password = "MilvusTest456!";

    let test_result: Result<()> = async {
        client
            .create_user(user.clone(), password.to_string())
            .await?;
        let users = client.list_users().await?;
        assert!(users.contains(&user));

        client
            .update_password(user.clone(), password.to_string(), new_password.to_string())
            .await?;

        client.create_role(role.clone()).await?;
        let roles = client.list_roles().await?;
        assert!(roles.contains(&role));

        client.grant_role(user.clone(), role.clone()).await?;
        let user_info = client.describe_user(user.clone()).await?;
        assert!(user_info
            .get(&user)
            .is_some_and(|roles| roles.contains(&role)));

        client
            .grant_privilege(
                role.clone(),
                "Load".to_string(),
                "Collection".to_string(),
                "*".to_string(),
                Some("default".to_string()),
            )
            .await?;
        client
            .grant_privilege_v2(
                role.clone(),
                "CreateCollection".to_string(),
                "*".to_string(),
                Some("default".to_string()),
            )
            .await?;

        let role_info = client.describe_role(role.clone()).await?;
        assert_eq!(
            role_info.get("role").and_then(|value| value.as_str()),
            Some(role.as_str())
        );

        client
            .revoke_privilege_v2(
                role.clone(),
                "CreateCollection".to_string(),
                "*".to_string(),
                Some("default".to_string()),
            )
            .await?;
        client
            .revoke_privilege(
                role.clone(),
                "Collection".to_string(),
                "Load".to_string(),
                "*".to_string(),
                Some("default".to_string()),
            )
            .await?;

        client.create_privilege_group(group.clone()).await?;
        client
            .add_privilege_to_group(group.clone(), vec!["Load".to_string()])
            .await?;
        let groups = client.list_privilege_groups().await?;
        assert!(groups
            .get(&group)
            .is_some_and(|privileges| privileges.contains(&"Load".to_string())));

        Ok(())
    }
    .await;

    let mut cleanup_error = None;
    if let Err(error) = client
        .revoke_privilege_from_group(group.clone(), vec!["Load".to_string()])
        .await
    {
        if cleanup_error.is_none() {
            cleanup_error = Some(error);
        }
    }
    if let Err(error) = client.drop_privilege_group(group).await {
        if cleanup_error.is_none() {
            cleanup_error = Some(error);
        }
    }
    if let Err(error) = client.revoke_role(user.clone(), role.clone()).await {
        if cleanup_error.is_none() {
            cleanup_error = Some(error);
        }
    }
    if let Err(error) = client.drop_role(role, false).await {
        if cleanup_error.is_none() {
            cleanup_error = Some(error);
        }
    }
    if let Err(error) = client.drop_user(user).await {
        if cleanup_error.is_none() {
            cleanup_error = Some(error);
        }
    }

    test_result?;
    if let Some(error) = cleanup_error {
        return Err(error);
    }

    Ok(())
}
