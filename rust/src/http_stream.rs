use std::io::{Read, Seek, SeekFrom};
use reqwest::blocking::Client;
use symphonia::core::io::MediaSource;

pub struct HttpStream {
   pub url: reqwest::Url,
   pub client: Client,
   pub response: Option<reqwest::blocking::Response>,
   pub pos: u64,
   pub len: Option<u64>,
}

impl HttpStream {
    pub fn new(url_str: &str) -> Result<Self, String> {
        let client = Client::new();
        let url = reqwest::Url::parse(url_str).map_err(|e| format!("Invalid URL: {e}"))?;

        let len = client
            .head(url.clone())
            .send()
            .ok()
            .and_then(|res| {
                res.headers()
                    .get(reqwest::header::CONTENT_LENGTH)
                    .and_then(|val| val.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
            });

        Ok(Self {
            url,
            client,
            response: None,
            pos: 0,
            len,
        })
    }

   pub fn send_range_request(&mut self, start: u64) -> Result<(), std::io::Error> {
        let range = format!("bytes={}-", start);
        let res = self.client.get(self.url.clone())
            .header(reqwest::header::RANGE, range)
            .send()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?
            .error_for_status()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        self.response = Some(res);
        self.pos = start;
        Ok(())
    }
}

impl Read for HttpStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.response.is_none() {
            self.send_range_request(self.pos)?;
        }

        match self.response.as_mut() {
            Some(res) => {
                let bytes_read = res.read(buf)?;
                if bytes_read == 0 {
                    // End of stream
                    self.response = None;
                } else {
                    self.pos += bytes_read as u64;
                }
                Ok(bytes_read)
            }
            None => Ok(0),
        }
    }
}

impl Seek for HttpStream {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(p) => p,
            SeekFrom::End(p) => {
                if let Some(len) = self.len {
                    len.checked_add_signed(p).ok_or_else(|| {
                      std::io::Error::new(std::io::ErrorKind::InvalidInput, "Seek underflow")
                    })?
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Unsupported,
                        "Seek from end not supported without content length",
                    ));
                }
            }
             SeekFrom::Current(p) => self.pos.checked_add_signed(p).ok_or_else(|| {
               std::io::Error::new(std::io::ErrorKind::InvalidInput, "Seek underflow")
          })?,
        };

        if new_pos > self.len.unwrap_or(u64::MAX) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Seek beyond end of stream",
            ));
        }

        self.send_range_request(new_pos)?;
        Ok(new_pos)
    }
}

impl MediaSource for HttpStream {
    fn is_seekable(&self) -> bool {
        self.len.is_some()
    }

    fn byte_len(&self) -> Option<u64> {
        self.len
    }
}