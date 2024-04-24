use socket_client::app_thread_and_poll;

pub mod main_all_poll;

fn main() -> std::io::Result<()> {
    app_thread_and_poll()?;
    //main_all_poll::app_all_poll()?;
    Ok(())
}