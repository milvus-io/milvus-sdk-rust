use milvus::client::{Client, ClientBuilder};
use milvus::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let client = ClientBuilder::new("http://localhost:19530")
        .username("root")
        .password("Milvus")
        .build()
        .await?;
    //user test
    user_test(&client).await?;
    //role test
    role_test(&client).await?;
    //privilege test
    privilege_test(&client).await?;
    Ok(())
}

async fn user_test(client: &Client) -> Result<()> {
    let user_name_a = "test_user_A";
    let user_name_b = "test_user_B";
    let password = "test_password";
    let new_password = "new_password";
    let old_password = "test_password";
    println!("========== Start user test ==========");
    //create user
    client.create_user(user_name_a, password).await?;
    client.create_user(user_name_b, password).await?;
    let res = client.list_users().await?;
    println!("After create users: {:?}", res);
    //describe user
    let res = client.describe_user(user_name_a).await?;
    println!("Describe user_a: {:?}", res);
    //update password
    client
        .update_password(user_name_a, old_password, new_password)
        .await?;
    //describe user
    let res = client.describe_user(user_name_a).await?;
    println!("After update password: {:?}", res);
    //drop user
    client.drop_user(user_name_a).await?;
    client.drop_user(user_name_b).await?;
    let res = client.list_users().await?;
    println!("After drop users: {:?}", res);
    println!("========== End user test ==========\n");
    Ok(())
}

async fn role_test(client: &Client) -> Result<()> {
    let user_name_a = "test_user_a";
    let role_name_a = "test_role_A";
    let role_name_b = "test_role_B";
    let password = "test_password";
    println!("========== Start role test ==========");
    //create role
    client.create_role(role_name_a).await?;
    client.create_role(role_name_b).await?;
    let res = client.list_roles().await?;
    println!("After create roles: {:?}", res);
    //describe role
    let res = client.describe_role(role_name_a).await?;
    println!("Describe role_a: {:?}", res);
    //grant role
    client.create_user(user_name_a, password).await?;
    client.grant_role(user_name_a, role_name_a).await?;
    let res = client.describe_user(user_name_a).await?;
    println!("Grant role to user: {:?}", res);
    //revoke role
    client.revoke_role(user_name_a, role_name_a).await?;
    let res = client.describe_user(user_name_a).await?;
    println!("Revoke role from user: {:?}", res);
    //drop role
    client.drop_role(role_name_a, true).await?;
    client.drop_role(role_name_b, true).await?;
    let res = client.list_roles().await?;
    println!("After drop roles: {:?}", res);
    //drop user
    client.drop_user(user_name_a).await?;
    println!("========== End role test ==========\n");
    Ok(())
}

async fn privilege_test(client: &Client) -> Result<()> {
    let privilege_group_name = "test_privilege_group";
    let privilege_name = "ShowCollections";
    let role_name = "test_role";
    let user_name = "test_user";
    let password = "test_password";
    println!("========== Start privilege test ==========");
    //create privilege group
    if client
        .list_privilege_groups()
        .await?
        .contains_key(&privilege_group_name.to_string())
    {
        client.drop_privilege_group(privilege_group_name).await?;
    }
    client.create_privilege_group(privilege_group_name).await?;
    let res = client.list_privilege_groups().await?;
    println!("After create privilege group: {:#?}", res);
    //create role
    if client.list_roles().await?.contains(&role_name.to_string()) {
        client.drop_role(role_name, true).await?;
    }
    client.create_role(role_name).await?;
    //create user
    if client.list_users().await?.contains(&user_name.to_string()) {
        client.drop_user(user_name).await?;
    }
    client.create_user(user_name, password).await?;
    //grant privilege
    client
        .grant_privilege(role_name, privilege_name, "Global", "*", None)
        .await?;
    let res = client.describe_role(role_name).await?;
    println!("After grant privilege: {:#?}", res);
    //add privilege to group
    client
        .add_privilege_to_group(privilege_group_name, vec![privilege_name.to_string()])
        .await?;
    let res = client.list_privilege_groups().await?;
    println!("After add privilege to group: {:#?}", res);
    //revoke privilege from group
    client
        .revoke_privilege_from_group(privilege_group_name, vec![privilege_name.to_string()])
        .await?;
    let res = client.list_privilege_groups().await?;
    println!("After revoke privilege from group: {:#?}", res);
    //revoke privilege
    client
        .revoke_privilege(role_name, "Global", "ShowCollections", "*", None)
        .await?;
    let res = client.describe_role(role_name).await?;
    println!("After revoke privilege: {:#?}", res);
    //drop privilege group
    client.drop_privilege_group(privilege_group_name).await?;
    let res = client.list_privilege_groups().await?;
    println!("After drop privilege group: {:#?}", res);
    //drop role
    client.drop_role(role_name, true).await?;
    client.drop_user(user_name).await?;
    println!("========== End privilege test ==========\n");
    Ok(())
}
