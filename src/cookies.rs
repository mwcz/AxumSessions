use crate::AxumSessionConfig;
use cookie::{Cookie, CookieJar, Key};
use http::{
    self,
    header::{COOKIE, SET_COOKIE},
    HeaderMap,
};

pub(crate) enum CookieType {
    Storable,
    Data,
}

impl CookieType {
    #[inline]
    pub(crate) fn get_name(&self, config: &AxumSessionConfig) -> String {
        match self {
            CookieType::Data => config.cookie_name.to_string(),
            CookieType::Storable => config.storable_cookie_name.to_string(),
        }
    }

    #[inline]
    pub(crate) fn get_age(&self, config: &AxumSessionConfig) -> Option<chrono::Duration> {
        match self {
            CookieType::Data => config.cookie_max_age,
            CookieType::Storable => config.storable_cookie_max_age,
        }
    }
}

pub(crate) trait CookiesExt {
    fn get_cookie(&self, name: &str, key: &Option<Key>) -> Option<Cookie<'static>>;
    fn add_cookie(&mut self, cookie: Cookie<'static>, key: &Option<Key>);
}

impl CookiesExt for CookieJar {
    fn get_cookie(&self, name: &str, key: &Option<Key>) -> Option<Cookie<'static>> {
        if let Some(key) = key {
            self.private(key).get(name)
        } else {
            self.get(name).cloned()
        }
    }

    fn add_cookie(&mut self, cookie: Cookie<'static>, key: &Option<Key>) {
        if let Some(key) = key {
            self.private_mut(key).add(cookie)
        } else {
            self.add(cookie)
        }
    }
}

pub(crate) fn create_cookie<'a>(
    config: &AxumSessionConfig,
    value: String,
    cookie_type: CookieType,
) -> Cookie<'a> {
    let mut cookie_builder = Cookie::build(cookie_type.get_name(config), value)
        .path(config.cookie_path.clone())
        .secure(config.cookie_secure)
        .http_only(config.cookie_http_only);

    if let Some(domain) = &config.cookie_domain {
        cookie_builder = cookie_builder
            .domain(domain.clone())
            .same_site(config.cookie_same_site);
    }

    if let Some(max_age) = cookie_type.get_age(config) {
        let time_duration = max_age.to_std().expect("Max Age out of bounds");
        cookie_builder =
            cookie_builder.max_age(time_duration.try_into().expect("Max Age out of bounds"));
    }

    cookie_builder.finish()
}

pub(crate) fn get_cookies(headers: &mut HeaderMap) -> CookieJar {
    let mut jar = CookieJar::new();

    let cookie_iter = headers
        .get_all(COOKIE)
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok());

    for cookie in cookie_iter {
        jar.add_original(cookie);
    }

    jar
}

pub(crate) fn set_cookies(jar: CookieJar, headers: &mut HeaderMap) {
    for cookie in jar.delta() {
        if let Ok(header_value) = cookie.encoded().to_string().parse() {
            headers.append(SET_COOKIE, header_value);
        }
    }
}
