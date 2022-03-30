use crate::codec::Codec;
use crate::device;
use crate::opts::DeviceOpts;
use crate::types::*;
use electricui_embedded::{decoder::Decoder as EUiDecoder, prelude::*};
use futures::stream::StreamExt;
use futures::SinkExt;
use thiserror::Error;
use tokio_util::codec::Framed;
use tracing::info;

pub async fn check(opts: DeviceOpts) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let dev = device::new(&opts)?;
    let mut enc_buf = vec![0_u8; Packet::<&[u8]>::MAX_PACKET_SIZE];
    let mut dec_buf = Box::new([0_u8; Packet::<&[u8]>::MAX_PACKET_SIZE]);
    let mut codec = Framed::new(dev, Codec::new(EUiDecoder::new(&mut dec_buf)));

    info!("Requesting board ID");
    let mut pkt = Packet::new_unchecked(&mut enc_buf);
    BoardId::encode_request(&mut pkt)?;
    codec.send(pkt).await?;

    let pkt = codec.next().await.ok_or(EndOfStreamError)??;
    let board_id = BoardId::decode_response(&pkt)?;
    println!("Board ID: 0x{:04X}", board_id);

    info!("Requesting board name");
    let mut pkt = Packet::new_unchecked(&mut enc_buf);
    BoardName::encode_request(&mut pkt)?;
    codec.send(pkt).await?;

    let pkt = codec.next().await.ok_or(EndOfStreamError)??;
    let board_name = BoardName::decode_response(&pkt)?;
    println!("Board name: {}", board_name);

    info!("Requesting writable IDs announcement");
    let mut pkt = Packet::new_unchecked(&mut enc_buf);
    WritableIdsAnnouncement::encode_request(&mut pkt)?;
    codec.send(pkt).await?;

    // TODO - up to 4 max-len per packet, could be arbitrary num packets
    // only read 1 for now
    let pkt = codec.next().await.ok_or(EndOfStreamError)??;
    let ids = IdsAnnouncement::decode_response(&pkt)?;
    println!("Message IDs ({}):", ids.len());
    for id in ids.as_slice().iter() {
        println!("  {}", id);
    }
    let pkt = codec.next().await.ok_or(EndOfStreamError)??;
    let num_ids: usize = WritableIdsAnnouncementEndList::decode_response(&pkt)?.into();
    println!("IDs count: {}", num_ids);

    info!("Requesting tracked variables");
    let mut pkt = Packet::new_unchecked(&mut enc_buf);
    TrackedVariables::encode_request(&mut pkt)?;
    codec.send(pkt).await?;

    let mut tracked_vars = TrackedVariables::default();
    for _ in 0..num_ids {
        let pkt = codec.next().await.ok_or(EndOfStreamError)??;
        tracked_vars.decode_response_accumulating(&pkt)?;
    }
    println!("Variables:");
    for var in tracked_vars.as_slice().iter() {
        println!("  {}", var);
    }

    let hb = Heartbeat::from(5);
    info!("Sending heartbeat {hb}");
    let mut pkt = Packet::new_unchecked(&mut enc_buf);
    hb.encode_request(&mut pkt)?;
    codec.send(pkt).await?;

    let pkt = codec.next().await.ok_or(EndOfStreamError)??;
    let hb_ack = Heartbeat::decode_response(&pkt)?;
    println!("Heartbeat: {}, matches: {}", hb_ack, hb == hb_ack);

    Ok(())
}

#[derive(Debug, Error)]
#[error("Encountered end of stream unexpectedly")]
pub struct EndOfStreamError;
