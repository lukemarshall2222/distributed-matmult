use hydro_deploy::Deployment; // imports Deployment struct from hydro_deploy crate

#[tokio::main] // marks the `main` function as the entry point for a Tokio runtime, enabling async ops
async fn main() {
    let mut deployment = Deployment::new(); // creates a new mutable Defployment instance, manages the deployment of processes

    let flow = hydro_lang::FlowBuilder::new(); // new FlowBuilder instance, used to define the structure of hydro flow
    let leader = flow.process(); // defines single leader process within the flow
    let workers = flow.cluster(); // defines a clusdter of worker processes within flow
    hydro_template::cluster_matmult::cluster_matmult(&leader, &workers); 
    // calls cluster_matmult function from the `hydro_template` crate, which is just src,
    // passing leader and worker cluster to set up the distributed matrix multiplication

    let _nodes = flow // starts defining how flow's processes and clusters will be deployed
        .with_process(&leader, deployment.Localhost()) // deploys leader psrocess to localhost
        .with_cluster(&workers, vec![deployment.Localhost(); 4]) // deploys the workers cluster, wi;lth 4 worker processes, all on localhost
        .deploy(&mut deployment); // triggers the deployment of the defined flow onto the deployment instance.
                                 // would normally y hold references to tsfdhe deployed nodes, but it's unused here.

    deployment.run_ctrl_c().await.unwrap(); // truns the deployed processes, keeps the deployment running until
                                            // a ctrl-c is rceceived. `.await` makes it an asynchronous call, and `.unwrap()` propogates the error                                   // handles potential errors during execution.
}