//! This is a very simple server for OPC UA.
use std::sync::Arc;
use log::warn;
use opcua::server::address_space::VariableBuilder;
use opcua::server::node_manager::memory::{
    simple_node_manager, InMemoryNodeManager, NamespaceMetadata, SimpleNodeManager,SimpleNodeManagerImpl,};
use opcua::server::{ServerBuilder, SubscriptionCache};
use opcua::types::{BuildInfo, DateTime, DataValue, NodeId, DataTypeId, StatusCode, Variant};

#[tokio::main]
async fn main() {
    println!("Hello, world of OPC UA!");
    opcua::console_logging::init();

    // Create an OPC UA server with sample configuration and default node set
    let (server, handle) = ServerBuilder::new()
        .with_config_from("../server.conf")
        .build_info(BuildInfo {
            product_uri: "https://github.com/freeopcua/async-opcua".into(),
            manufacturer_name: "Rust OPC-UA".into(),
            product_name: "Rust OPC-UA sample server".into(),
            // Here you could use something to inject the build time, version, number at compile time
            software_version: "0.1.0".into(),
            build_number: "1".into(),
            build_date: DateTime::now(),
        })
        .with_node_manager(simple_node_manager(
            NamespaceMetadata {
                namespace_uri: "urn:SimpleServer".to_owned(),
                ..Default::default()
            },
            "simple",
        ))
        .trust_client_certs(true)
        .build()
        .unwrap();
    
    let node_manager = handle
        .node_managers()
        .get_of_type::<SimpleNodeManager>()
        .unwrap();
    
    let ns = handle.get_namespace_index("urn:SimpleServer").unwrap();
unsafe{
    add_example_variable(ns, node_manager, handle.subscriptions().clone());
}
    // If you don't register a ctrl-c handler, the server will close without informing clients.
    let handle_c = handle.clone();
    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            warn!("Failed to register CTRL-C handler: {e}");
            return;
        }
        handle_c.cancel();
    });

    // Run the server. This does not ordinarily exit so you must Ctrl+C to terminate
    server.run().await.unwrap();
}

unsafe fn add_example_variable(
    ns: u16,
    manager: Arc<InMemoryNodeManager<SimpleNodeManagerImpl>>,
    subscriptions: Arc<SubscriptionCache>,) 
{
    // These will be the node ids of the new variables
    let v1_node = NodeId::new(ns, "v1");
    let address_space = manager.address_space();

    // The address space is guarded so obtain a lock to change it
    let mut address_spacel = address_space.write();

    // Create a sample folder under objects folder
    let sample_folder_id = NodeId::new(ns, "folder");
    address_spacel.add_folder(
        &sample_folder_id,
        "Sample",
        "Sample",
        &NodeId::objects_folder_id(),
    );

    VariableBuilder::new(&v1_node, "v1", "v1")
        .data_type(DataTypeId::Int32)
        .value(1)
        .writable()
        .organized_by(&sample_folder_id)
        .insert(&mut *address_spacel);

    // The simple node manager lets you set dynamic getters:
    let mgr_ref = manager.clone();

    //address_space.force_unlock_write();
    //manager
    //.set_value(&subscriptions, &v1_node, None, DataValue::new_now(42))
    //.unwrap();

    //manager
    //.inner()
    //.add_read_callback(v1_node.clone(), move |_, _, _| {
    //    Ok(DataValue::new_now(42))
    //});
    
    // If you will remove callback, then BadNotWritable raised on Client side
    manager
        .inner()
        .add_write_callback(v1_node.clone(),  move |v, _| {
            println!("Write callback - begin");
            if let Some(Variant::Int32(val)) = v.value {
                println!("New Value: {}", val);
                DataValue::new_now(val);
                mgr_ref // Issue is write lock in .set_value:
                    .set_value(&subscriptions, &v1_node, None, v)
                    .unwrap();
                println!("Write callback - end");
                StatusCode::Good
            } else {
                StatusCode::BadTypeMismatch
            }
        });

}