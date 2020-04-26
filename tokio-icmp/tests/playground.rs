use tokio_icmp::ICMPV4Socket;

#[tokio::test]
async fn it_works() -> Result<(), Box<dyn std::error::Error>> {
    let socket = ICMPV4Socket::new()?;

    let mut buffer = [0u8; 1500];
    let (bytes_read, from) = socket.recv_from(&mut buffer).await?;

    // it seems this is the ipv4 packet too
    println!("{:?} {:?}", from, &buffer[0..bytes_read]);

    Ok(())
}
