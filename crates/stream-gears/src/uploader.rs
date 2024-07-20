use anyhow::{Context, Result};
use biliup::bilibili::Vid::Bvid;
use biliup::client::StatelessClient;
use biliup::error::Kind;
use biliup::uploader::bilibili::{Credit, ResponseData, Studio};
use biliup::uploader::credential::login_by_cookies;
use biliup::uploader::line::Probe;
use biliup::uploader::{line, VideoFile};
use futures::StreamExt;
use pyo3::prelude::*;
use pyo3::pyclass;

use std::path::PathBuf;
use std::time::Instant;
use tracing::info;

use typed_builder::TypedBuilder;

#[pyclass]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UploadLine {
    Bda2,
    Ws,
    Qn,
    // Kodo,
    // Cos,
    // CosInternal,
    Bldsa,
    Tx,
    Txa,
    Bda,
}

#[derive(FromPyObject)]
pub struct PyCredit {
    #[pyo3(item("type"))]
    type_id: i8,
    #[pyo3(item("raw_text"))]
    raw_text: String,
    #[pyo3(item("biz_id"))]
    biz_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct StudioPre {
    video_path: Vec<PathBuf>,
    cookie_file: PathBuf,
    line: Option<UploadLine>,
    limit: usize,
    title: String,
    tid: u16,
    tag: String,
    copyright: u8,
    source: String,
    desc: String,
    dynamic: String,
    cover: String,
    dtime: Option<u32>,
    dolby: u8,
    lossless_music: u8,
    no_reprint: u8,
    open_elec: u8,
    #[builder(default = false)]
    up_close_reply: bool,
    #[builder(default = false)]
    up_selection_reply: bool,
    #[builder(default = false)]
    up_close_danmu: bool,
    desc_v2_credit: Vec<PyCredit>,
}

pub async fn upload2(
    studio_pre: StudioPre,
    by_app: bool,
    proxy: Option<&str>,
    user_agent: Option<&str>,
) -> Result<ResponseData> {
    // let file = std::fs::File::options()
    //     .read(true)
    //     .write(true)
    //     .open(&cookie_file);
    let StudioPre {
        video_path,
        cookie_file,
        line,
        limit,
        title,
        tid,
        tag,
        copyright,
        source,
        desc,
        dynamic,
        cover,
        dtime,
        dolby,
        lossless_music,
        no_reprint,
        open_elec,
        up_close_reply,
        up_selection_reply,
        up_close_danmu,
        desc_v2_credit,
    } = studio_pre;

    let bilibili = login_by_cookies(&cookie_file).await;
    let bilibili = if let Err(Kind::IO(_)) = bilibili {
        bilibili
            .with_context(|| String::from("open cookies file: ") + &cookie_file.to_string_lossy())?
    } else {
        bilibili?
    };

    let client = StatelessClient::default();
    let mut videos = Vec::new();
    let line = match line {
        Some(UploadLine::Bda2) => line::bda2(),
        Some(UploadLine::Ws) => line::ws(),
        Some(UploadLine::Qn) => line::qn(),
        // Some(UploadLine::Kodo) => line::kodo(),
        // Some(UploadLine::Cos) => line::cos(),
        // Some(UploadLine::CosInternal) => line::cos_internal(),
        Some(UploadLine::Bda) => line::bda(),
        Some(UploadLine::Tx) => line::tx(),
        Some(UploadLine::Txa) => line::txa(),
        Some(UploadLine::Bldsa) => line::bldsa(),
        None => Probe::probe(&client.client).await.unwrap_or_default(),
    };
    for video_path in video_path {
        println!("{:?}", video_path.canonicalize()?.to_str());
        info!("{line:?}");
        let video_file = VideoFile::new(&video_path)?;
        let total_size = video_file.total_size;
        let file_name = video_file.file_name.clone();
        let uploader = line.pre_upload(&bilibili, video_file).await?;

        let instant = Instant::now();

        let video = uploader
            .upload(client.clone(), limit, |vs| {
                vs.map(|vs| {
                    let chunk = vs?;
                    let len = chunk.len();
                    Ok((chunk, len))
                })
            })
            .await?;
        let t = instant.elapsed().as_millis();
        info!(
            "Upload completed: {file_name} => cost {:.2}s, {:.2} MB/s.",
            t as f64 / 1000.,
            total_size as f64 / 1000. / t as f64
        );
        videos.push(video);
    }

    let mut desc_v2 = Vec::new();
    for credit in desc_v2_credit {
        desc_v2.push(Credit {
            type_id: credit.type_id,
            raw_text: credit.raw_text,
            biz_id: credit.biz_id,
        });
    }

    let mut studio: Studio = Studio::builder()
        .desc(desc)
        .dtime(dtime)
        .copyright(copyright)
        .cover(cover)
        .dynamic(dynamic)
        .source(source)
        .tag(tag)
        .tid(tid)
        .title(title)
        .videos(videos)
        .dolby(dolby)
        .lossless_music(lossless_music)
        .no_reprint(no_reprint)
        .open_elec(open_elec)
        .up_close_reply(up_close_reply)
        .up_selection_reply(up_selection_reply)
        .up_close_danmu(up_close_danmu)
        .desc_v2(Some(desc_v2))
        .build();

    if !studio.cover.is_empty() {
        let url = bilibili
            .cover_up(
                &std::fs::read(&studio.cover)
                    .with_context(|| format!("cover: {}", studio.cover))?,
            )
            .await?;
        println!("{url}");
        studio.cover = url;
    }

    let response = match by_app {
        true => bilibili.submit_by_app(&studio, proxy, user_agent).await?,
        false => bilibili.submit(&studio).await?,
    };

    Ok(response)
}

pub async fn fetch(cookie_file: &PathBuf, bvid: &str) -> Result<String> {
    let bilibili = login_by_cookies(&cookie_file).await?;
    let archive = bilibili.video_data(&Bvid(bvid.to_owned())).await?;
    Ok(serde_json::to_string(&archive)?)
}

pub async fn edit(
    cookie_file: &PathBuf,
    bvid: &str,
    title: Option<&str>,
    cover: Option<&str>,
    tag: Option<&str>,
) -> Result<serde_json::Value> {
    let bilibili = login_by_cookies(&cookie_file).await?;
    let mut studio = bilibili.studio_data(&Bvid(bvid.to_owned())).await?;

    if let Some(title) = title {
        studio.title = title.to_owned();
    }

    if let Some(cover) = cover {
        let input = tokio::fs::read(&cover).await?;
        let url = bilibili.cover_up(&input).await?;
        studio.cover = url;
    }

    if let Some(tag) = tag {
        studio.tag = tag.to_owned();
    }

    let response = bilibili.edit(&studio).await?;

    if response["code"] != 0 {
        return Err(anyhow::anyhow!("Edit failed: {}", response));
    }

    Ok(response)
}

pub async fn archives(cookie_file: &PathBuf, status: &str, page: u32) -> Result<serde_json::Value> {
    let bilibili = login_by_cookies(&cookie_file).await?;
    let response = bilibili.archives(status, page).await?;
    Ok(response)
}

pub async fn delete(cookie_file: &PathBuf, bvid: &str) -> Result<()> {
    let bilibili = login_by_cookies(&cookie_file).await?;
    bilibili.delete_by_app(bvid).await?;
    Ok(())
}
