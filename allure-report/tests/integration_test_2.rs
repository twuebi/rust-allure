// mod some;
//
// #[cfg(test)]
// mod test {
//     use crate::some::{Server, Test};
//     use reqwest::{Client, Method};
//     use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
//     use std::net::SocketAddr;
//     use std::path::PathBuf;
//     use tokio::sync::mpsc::UnboundedSender;
//     use untitled::middleware::LoggingMiddleware;
//     use untitled::{Message, Reporter, Status};
//
//     #[tokio::test]
//     async fn test() {
//         let server = Server::new(3000);
//         let addr = server.addr.clone();
//         tokio::task::spawn(async {
//             server.serve().await;
//             eprintln!("Exit server");
//         });
//         let reqwest_client = Client::builder().build().unwrap();
//         let (reporter, mut tx, result_rx) =
//             Reporter::new("Test it".into(), "Test that".into(), module_path!());
//         let task_handle = tokio::task::spawn(reporter.task());
//
//         let client = ClientBuilder::new(reqwest_client)
//             .with(LoggingMiddleware::new(PathBuf::from("allure-results"), tx.clone()).await)
//             .build();
//
//         step_1(addr, &client, &mut tx).await;
//         step_2(addr, client, &mut tx).await;
//
//         tx.send(Message::Result).unwrap();
//         let result = result_rx.await.unwrap();
//         Reporter::write_result(&result, "allure-results".into()).await;
//     }
//
//     async fn step_2(
//         addr: SocketAddr,
//         client: ClientWithMiddleware,
//         tx: &mut UnboundedSender<Message>,
//     ) {
//         tx.send(Message::StartStep("Test it works with json".into()))
//             .unwrap();
//
//         let builder = client
//             .request(Method::GET, format!("http://{}/json", addr))
//             .json(&Test {
//                 a: "XYZ".to_string(),
//             });
//         let _res = client.execute(builder.build().unwrap()).await.unwrap();
//         tx.send(Message::FinalizeStep(Status::Failed)).unwrap();
//     }
//
//     async fn step_1(
//         addr: SocketAddr,
//         client: &ClientWithMiddleware,
//         tx: &mut UnboundedSender<Message>,
//     ) {
//         tx.send(Message::StartStep("Test it works".into())).unwrap();
//
//         let builder = client.request(Method::GET, format!("http://{}/", addr));
//         let _res = client.execute(builder.build().unwrap()).await.unwrap();
//
//         tx.send(Message::FinalizeStep(Status::Passed)).unwrap();
//     }
//
//     #[tokio::test]
//     async fn test2() {
//         let server = Server::new(3003);
//         let addr = server.addr.clone();
//         tokio::task::spawn(async {
//             server.serve().await;
//             eprintln!("Exit server");
//         });
//         let reqwest_client = Client::builder().build().unwrap();
//         let (reporter, mut tx, result_rx) =
//             Reporter::new("Test it2".into(), "Test that2".into(), module_path!());
//         let task_handle = tokio::task::spawn(reporter.task());
//
//         let client = ClientBuilder::new(reqwest_client)
//             .with(LoggingMiddleware::new(PathBuf::from("allure-results"), tx.clone()).await)
//             .build();
//
//         step_1(addr, &client, &mut tx).await;
//         step_2(addr, client, &mut tx).await;
//
//         tx.send(Message::Result).unwrap();
//         let result = result_rx.await.unwrap();
//         Reporter::write_result(&result, "allure-results".into()).await;
//     }
//
//     #[tokio::test]
//     async fn test3() {
//         let server = Server::new(3002);
//         let addr = server.addr.clone();
//         tokio::task::spawn(async {
//             server.serve().await;
//             eprintln!("Exit server");
//         });
//         let reqwest_client = Client::builder().build().unwrap();
//         let (reporter, mut tx, result_rx) =
//             Reporter::new("Test it3".into(), "Test that3".into(), module_path!());
//         let _task_handle = tokio::task::spawn(reporter.task());
//
//         let client = ClientBuilder::new(reqwest_client)
//             .with(LoggingMiddleware::new(PathBuf::from("allure-results"), tx.clone()).await)
//             .build();
//
//         step_1(addr, &client, &mut tx).await;
//         step_2(addr, client, &mut tx).await;
//
//         tx.send(Message::Result).unwrap();
//         let result = result_rx.await.unwrap();
//         Reporter::write_result(&result, "allure-results".into()).await;
//     }
// }
