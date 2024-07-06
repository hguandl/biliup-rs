mod login;
mod uploader;

use pyo3::marker::Ungil;
use pyo3::prelude::*;
use std::future::Future;
use time::macros::format_description;
use uploader::{PyCredit, StudioPre};

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::uploader::UploadLine;
use biliup::credential::Credential;
use biliup::downloader::construct_headers;
use biliup::downloader::extractor::CallbackFn;
use biliup::downloader::util::Segmentable;
use tracing_subscriber::layer::SubscriberExt;

#[derive(FromPyObject)]
pub enum PySegment {
    Time {
        #[pyo3(attribute("time"))]
        time: u64,
    },
    Size {
        #[pyo3(attribute("size"))]
        size: u64,
    },
}

#[pyfunction]
fn download(
    py: Python<'_>,
    url: &str,
    header_map: HashMap<String, String>,
    file_name: &str,
    segment: PySegment,
) -> PyResult<()> {
    download_with_callback(py, url, header_map, file_name, segment, None)
}

#[pyfunction]
fn download_with_callback(
    py: Python<'_>,
    url: &str,
    header_map: HashMap<String, String>,
    file_name: &str,
    segment: PySegment,
    file_name_callback_fn: Option<PyObject>,
) -> PyResult<()> {
    py.allow_threads(|| {
        let map = construct_headers(header_map);
        // 输出到控制台中
        unsafe {
            time::util::local_offset::set_soundness(time::util::local_offset::Soundness::Unsound);
        }
        let local_time = tracing_subscriber::fmt::time::LocalTime::new(format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        ));
        let formatting_layer = tracing_subscriber::FmtSubscriber::builder()
            // will be written to stdout.
            // builds the subscriber.
            .with_timer(local_time.clone())
            .finish();
        let file_appender = tracing_appender::rolling::never("", "download.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        let file_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_timer(local_time)
            .with_writer(non_blocking);

        let segment = match segment {
            PySegment::Time { time } => Segmentable::new(Some(Duration::from_secs(time)), None),
            PySegment::Size { size } => Segmentable::new(None, Some(size)),
        };

        let file_name_hook = file_name_callback_fn.map(|callback_fn| -> CallbackFn {
            Box::new(move |fmt_file_name| {
                Python::with_gil(|py| match callback_fn.call1(py, (fmt_file_name,)) {
                    Ok(_) => {}
                    Err(_) => {
                        tracing::error!("Unable to invoke the callback function.")
                    }
                })
            })
        });

        let collector = formatting_layer.with(file_layer);
        tracing::subscriber::with_default(collector, || -> PyResult<()> {
            match biliup::downloader::download(url, map, file_name, segment, file_name_hook) {
                Ok(res) => Ok(res),
                Err(err) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "{}, {}",
                    err.root_cause(),
                    err
                ))),
            }
        })
    })
}

#[pyfunction]
fn login_by_cookies(file: String) -> PyResult<bool> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async { login::login_by_cookies(&file).await });
    match result {
        Ok(_) => Ok(true),
        Err(err) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
            "{}, {}",
            err.root_cause(),
            err
        ))),
    }
}

#[pyfunction]
fn send_sms(country_code: u32, phone: u64) -> PyResult<String> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async { login::send_sms(country_code, phone).await });
    match result {
        Ok(res) => Ok(res.to_string()),
        Err(err) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
            "{}",
            err
        ))),
    }
}

#[pyfunction]
fn login_by_sms(code: u32, ret: String) -> PyResult<bool> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result =
        rt.block_on(async { login::login_by_sms(code, serde_json::from_str(&ret).unwrap()).await });
    match result {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[pyfunction]
fn get_qrcode() -> PyResult<String> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async { login::get_qrcode().await });
    match result {
        Ok(res) => Ok(res.to_string()),
        Err(err) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
            "{}",
            err
        ))),
    }
}

#[pyfunction]
fn login_by_qrcode(ret: String) -> PyResult<String> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let info = Credential::new()
            .login_by_qrcode(serde_json::from_str(&ret).unwrap())
            .await?;
        let res = serde_json::to_string_pretty(&info)?;
        Ok::<_, anyhow::Error>(res)
    })
    .map_err(|err| pyo3::exceptions::PyRuntimeError::new_err(format!("{:#?}", err)))
}

#[pyfunction]
fn login_by_web_cookies(sess_data: String, bili_jct: String) -> PyResult<bool> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async { login::login_by_web_cookies(&sess_data, &bili_jct).await });
    match result {
        Ok(_) => Ok(true),
        Err(err) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
            "{}",
            err
        ))),
    }
}

#[pyfunction]
fn login_by_web_qrcode(sess_data: String, dede_user_id: String) -> PyResult<bool> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async { login::login_by_web_qrcode(&sess_data, &dede_user_id).await });
    match result {
        Ok(_) => Ok(true),
        Err(err) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
            "{}",
            err
        ))),
    }
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
fn upload(
    py: Python<'_>,
    video_path: Vec<PathBuf>,
    cookie_file: PathBuf,
    title: String,
    tid: u16,
    tag: String,
    copyright: u8,
    source: String,
    desc: String,
    dynamic: String,
    cover: String,
    dolby: u8,
    lossless_music: u8,
    no_reprint: u8,
    open_elec: u8,
    limit: usize,
    desc_v2: Vec<PyCredit>,
    dtime: Option<u32>,
    line: Option<UploadLine>,
) -> PyResult<()> {
    upload2(
        py,
        false,
        video_path,
        cookie_file,
        title,
        tid,
        tag,
        copyright,
        source,
        desc,
        dynamic,
        cover,
        dolby,
        lossless_music,
        no_reprint,
        open_elec,
        false,
        false,
        false,
        limit,
        desc_v2,
        dtime,
        line,
        None,
        None,
    )
    .map(|_| ())
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(signature = (video_path, cookie_file, title, tid=171, tag="".to_string(), copyright=2, source="".to_string(), desc="".to_string(), dynamic="".to_string(), cover="".to_string(), dolby=0, lossless_music=0, no_reprint=0, open_elec=0, up_close_reply=false, up_selection_reply=false, up_close_danmu=false, limit=3, desc_v2=vec![], dtime=None, line=None))]
fn upload_by_app(
    py: Python<'_>,
    video_path: Vec<PathBuf>,
    cookie_file: PathBuf,
    title: String,
    tid: u16,
    tag: String,
    copyright: u8,
    source: String,
    desc: String,
    dynamic: String,
    cover: String,
    dolby: u8,
    lossless_music: u8,
    no_reprint: u8,
    open_elec: u8,
    up_close_reply: bool,
    up_selection_reply: bool,
    up_close_danmu: bool,
    limit: usize,
    desc_v2: Vec<PyCredit>,
    dtime: Option<u32>,
    line: Option<UploadLine>,
) -> PyResult<()> {
    upload2(
        py,
        true,
        video_path,
        cookie_file,
        title,
        tid,
        tag,
        copyright,
        source,
        desc,
        dynamic,
        cover,
        dolby,
        lossless_music,
        no_reprint,
        open_elec,
        up_close_reply,
        up_selection_reply,
        up_close_danmu,
        limit,
        desc_v2,
        dtime,
        line,
        None,
        None,
    )
    .map(|_| ())
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
fn upload2(
    py: Python<'_>,
    by_app: bool,
    video_path: Vec<PathBuf>,
    cookie_file: PathBuf,
    title: String,
    tid: u16,
    tag: String,
    copyright: u8,
    source: String,
    desc: String,
    dynamic: String,
    cover: String,
    dolby: u8,
    lossless_music: u8,
    no_reprint: u8,
    open_elec: u8,
    up_close_reply: bool,
    up_selection_reply: bool,
    up_close_danmu: bool,
    limit: usize,
    desc_v2: Vec<PyCredit>,
    dtime: Option<u32>,
    line: Option<UploadLine>,
    proxy: Option<String>,
    user_agent: Option<String>,
) -> PyResult<String> {
    spawn_logged_task(py, || async {
        let studio_pre = StudioPre::builder()
            .video_path(video_path)
            .cookie_file(cookie_file)
            .line(line)
            .limit(limit)
            .title(title)
            .tid(tid)
            .tag(tag)
            .copyright(copyright)
            .source(source)
            .desc(desc)
            .dynamic(dynamic)
            .cover(cover)
            .dtime(dtime)
            .dolby(dolby)
            .lossless_music(lossless_music)
            .no_reprint(no_reprint)
            .open_elec(open_elec)
            .up_close_reply(up_close_reply)
            .up_selection_reply(up_selection_reply)
            .up_close_danmu(up_close_danmu)
            .desc_v2_credit(desc_v2)
            .build();

        match uploader::upload2(studio_pre, by_app, proxy.as_deref(), user_agent.as_deref()).await {
            Ok(value) => Ok(value.data.unwrap()["bvid"].as_str().unwrap().to_owned()),
            Err(err) => Err(pyerr_from_anyhow(err)),
        }
    })
}

#[pyfunction]
fn fetch(py: Python<'_>, cookie_file: PathBuf, bvid: String) -> PyResult<String> {
    spawn_logged_task(py, || async {
        let response = uploader::fetch(&cookie_file, &bvid).await;
        match response {
            Ok(json) => Ok(json),
            Err(err) => Err(pyerr_from_anyhow(err)),
        }
    })
}

#[pyfunction]
fn edit(
    py: Python<'_>,
    cookie_file: PathBuf,
    bvid: String,
    title: Option<String>,
    tag: Option<String>,
    cover: Option<String>,
) -> PyResult<()> {
    spawn_logged_task(py, || async {
        let result = uploader::edit(
            &cookie_file,
            bvid.as_str(),
            title.as_deref(),
            cover.as_deref(),
            tag.as_deref(),
        )
        .await;
        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(pyerr_from_anyhow(err)),
        }
    })
}

fn spawn_logged_task<T: Send, R, F>(py: Python<'_>, f: F) -> PyResult<T>
where
    R: Future<Output = PyResult<T>>,
    F: Ungil + Send + FnOnce() -> R,
{
    py.allow_threads(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        // 输出到控制台中
        unsafe {
            time::util::local_offset::set_soundness(time::util::local_offset::Soundness::Unsound);
        }
        let local_time = tracing_subscriber::fmt::time::LocalTime::new(format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        ));
        let formatting_layer = tracing_subscriber::FmtSubscriber::builder()
            // will be written to stdout.
            // builds the subscriber.
            .with_timer(local_time.clone())
            .finish();
        let file_appender = tracing_appender::rolling::never("", "upload.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        let file_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_timer(local_time)
            .with_writer(non_blocking);

        let collector = formatting_layer.with(file_layer);

        tracing::subscriber::with_default(collector, || rt.block_on(f()))
    })
}

fn pyerr_from_anyhow(err: anyhow::Error) -> PyErr {
    pyo3::exceptions::PyRuntimeError::new_err(format!("{}, {}", err.root_cause(), err))
}

/// A Python module implemented in Rust.
#[pymodule]
fn stream_gears(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // let file_appender = tracing_appender::rolling::daily("", "upload.log");
    // let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    // tracing_subscriber::fmt()
    //     .with_writer(non_blocking)
    //     .init();
    m.add_function(wrap_pyfunction!(upload, m)?)?;
    m.add_function(wrap_pyfunction!(upload_by_app, m)?)?;
    m.add_function(wrap_pyfunction!(upload2, m)?)?;
    m.add_function(wrap_pyfunction!(fetch, m)?)?;
    m.add_function(wrap_pyfunction!(edit, m)?)?;
    m.add_function(wrap_pyfunction!(download, m)?)?;
    m.add_function(wrap_pyfunction!(download_with_callback, m)?)?;
    m.add_function(wrap_pyfunction!(login_by_cookies, m)?)?;
    m.add_function(wrap_pyfunction!(send_sms, m)?)?;
    m.add_function(wrap_pyfunction!(login_by_qrcode, m)?)?;
    m.add_function(wrap_pyfunction!(get_qrcode, m)?)?;
    m.add_function(wrap_pyfunction!(login_by_sms, m)?)?;
    m.add_function(wrap_pyfunction!(login_by_web_cookies, m)?)?;
    m.add_function(wrap_pyfunction!(login_by_web_qrcode, m)?)?;
    m.add_class::<UploadLine>()?;
    Ok(())
}
