use tokio::io::{self, AsyncRead, AsyncWrite};
use tokio::net::TcpStream;

/// Janela maior para vencer RTT alto; o limite de bytes não enviados impede
/// que o ganho de throughput vire uma fila enorme durante o upload.
const RELAY_BUFFER_SIZE: usize = 128 * 1024;
const SOCKET_RECEIVE_BUFFER_SIZE: libc::c_int = 512 * 1024;
const SOCKET_SEND_BUFFER_SIZE: libc::c_int = 256 * 1024;
const TCP_NOTSENT_LOWAT_BYTES: libc::c_uint = 64 * 1024;
const TCP_KEEPIDLE_SECS: libc::c_int = 20;
const TCP_KEEPINTVL_SECS: libc::c_int = 10;
const TCP_KEEPCNT: libc::c_int = 6;

/// Ajustes seguros para túneis TCP interativos e transferências contínuas.
///
/// TCP_NODELAY reduz a espera do Nagle em SSH. Buffers maiores ajudam redes
/// móveis com janela TCP maior. TCP_QUICKACK é reaplicado no Linux quando
/// disponível; ele não é exigido para o funcionamento em outros sistemas.
pub fn tune_tcp_stream(stream: &TcpStream) {
    let _ = stream.set_nodelay(true);
    #[cfg(target_os = "linux")]
    unsafe {
        let fd = stream.as_raw_fd();
        let receive_size = std::mem::size_of_val(&SOCKET_RECEIVE_BUFFER_SIZE) as libc::socklen_t;
        let _ = libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_RCVBUF,
            &SOCKET_RECEIVE_BUFFER_SIZE as *const _ as *const libc::c_void,
            receive_size,
        );
        let send_size = std::mem::size_of_val(&SOCKET_SEND_BUFFER_SIZE) as libc::socklen_t;
        let _ = libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_SNDBUF,
            &SOCKET_SEND_BUFFER_SIZE as *const _ as *const libc::c_void,
            send_size,
        );
        let _ = libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_NOTSENT_LOWAT,
            &TCP_NOTSENT_LOWAT_BYTES as *const _ as *const libc::c_void,
            std::mem::size_of_val(&TCP_NOTSENT_LOWAT_BYTES) as libc::socklen_t,
        );
        // BBR reduz filas e costuma ser melhor em RTT alto; se não estiver
        // disponível no kernel, o erro é ignorado e o congestion control atual
        // continua valendo para esta conexão.
        let bbr = b"bbr\0";
        let _ = libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_CONGESTION,
            bbr.as_ptr() as *const libc::c_void,
            bbr.len() as libc::socklen_t,
        );
        let quickack: libc::c_int = 1;
        let _ = libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_QUICKACK,
            &quickack as *const _ as *const libc::c_void,
            std::mem::size_of_val(&quickack) as libc::socklen_t,
        );
        let keepalive: libc::c_int = 1;
        let _ = libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_KEEPALIVE,
            &keepalive as *const _ as *const libc::c_void,
            std::mem::size_of_val(&keepalive) as libc::socklen_t,
        );
        let _ = libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_KEEPIDLE,
            &TCP_KEEPIDLE_SECS as *const _ as *const libc::c_void,
            std::mem::size_of_val(&TCP_KEEPIDLE_SECS) as libc::socklen_t,
        );
        let _ = libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_KEEPINTVL,
            &TCP_KEEPINTVL_SECS as *const _ as *const libc::c_void,
            std::mem::size_of_val(&TCP_KEEPINTVL_SECS) as libc::socklen_t,
        );
        let _ = libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_KEEPCNT,
            &TCP_KEEPCNT as *const _ as *const libc::c_void,
            std::mem::size_of_val(&TCP_KEEPCNT) as libc::socklen_t,
        );
    }
}

/// Tuning conservador para HCR. O protocolo abre canais curtos e independentes
/// de upload/download; NOTSENT_LOWAT e BBR podem transformar backpressure em
/// encerramentos prematuros em algumas redes móveis, então não são aplicados.
pub fn tune_hcr_stream(stream: &TcpStream) {
    let _ = stream.set_nodelay(true);
    #[cfg(target_os = "linux")]
    unsafe {
        let fd = stream.as_raw_fd();
        let receive_size = std::mem::size_of_val(&SOCKET_RECEIVE_BUFFER_SIZE) as libc::socklen_t;
        let _ = libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_RCVBUF,
            &SOCKET_RECEIVE_BUFFER_SIZE as *const _ as *const libc::c_void,
            receive_size,
        );
        let send_size = std::mem::size_of_val(&SOCKET_SEND_BUFFER_SIZE) as libc::socklen_t;
        let _ = libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_SNDBUF,
            &SOCKET_SEND_BUFFER_SIZE as *const _ as *const libc::c_void,
            send_size,
        );
        let keepalive: libc::c_int = 1;
        let _ = libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_KEEPALIVE,
            &keepalive as *const _ as *const libc::c_void,
            std::mem::size_of_val(&keepalive) as libc::socklen_t,
        );
        let _ = libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_KEEPIDLE,
            &TCP_KEEPIDLE_SECS as *const _ as *const libc::c_void,
            std::mem::size_of_val(&TCP_KEEPIDLE_SECS) as libc::socklen_t,
        );
        let _ = libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_KEEPINTVL,
            &TCP_KEEPINTVL_SECS as *const _ as *const libc::c_void,
            std::mem::size_of_val(&TCP_KEEPINTVL_SECS) as libc::socklen_t,
        );
        let _ = libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_KEEPCNT,
            &TCP_KEEPCNT as *const _ as *const libc::c_void,
            std::mem::size_of_val(&TCP_KEEPCNT) as libc::socklen_t,
        );
        let quickack: libc::c_int = 1;
        let _ = libc::setsockopt(
            fd,
            libc::IPPROTO_TCP,
            libc::TCP_QUICKACK,
            &quickack as *const _ as *const libc::c_void,
            std::mem::size_of_val(&quickack) as libc::socklen_t,
        );
    }
}

pub async fn copy_bidirectional<A, B>(a: &mut A, b: &mut B) -> io::Result<(u64, u64)>
where
    A: AsyncRead + AsyncWrite + Unpin,
    B: AsyncRead + AsyncWrite + Unpin,
{
    tokio::io::copy_bidirectional_with_sizes(
        a,
        b,
        RELAY_BUFFER_SIZE,
        RELAY_BUFFER_SIZE,
    )
    .await
}

#[cfg(target_os = "linux")]
use std::os::fd::AsRawFd;
