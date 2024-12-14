#![forbid(unsafe_code)]
pub type SettingsModifier = Box<dyn FnOnce(&mut Settings)>;

pub enum ListMode {
    Set,
    Add,
    Del,
}

#[repr(u32)]
pub enum SettingKind {
    Flag(SettingsModifier) = crate::common::HARDENED_ENUM_VALUE_0,
    Integer(fn(&str) -> Option<SettingsModifier>) = crate::common::HARDENED_ENUM_VALUE_1,
    Text(fn(&str) -> Option<SettingsModifier>) = crate::common::HARDENED_ENUM_VALUE_2,
    List(fn(ListMode, Vec<String>) -> Option<SettingsModifier>) =
        crate::common::HARDENED_ENUM_VALUE_3,
}

mod settings_dsl;
use settings_dsl::{defaults, ifdef, modifier_of, negate_of, type_of, value_of};

defaults! {
    always_query_group_plugin = false
    always_set_home           = false
    env_reset                 = true
    mail_badpass              = true
    match_group_by_gid        = false
    use_pty                   = true
    visiblepw                 = false
    env_editor                = true

    passwd_tries              = 3 [0..=1000]

    secure_path               = None (!= None)

    verifypw                  = all (!= never) [all, always, any, never]

    timestamp_timeout         = (15*60) (!= 0) {fractional_minutes}

    env_keep                  = ["COLORS", "DISPLAY", "HOSTNAME", "KRB5CCNAME", "LS_COLORS", "PATH",
                                 "PS1", "PS2", "XAUTHORITY", "XAUTHORIZATION", "XDG_CURRENT_DESKTOP"]

    env_check                 = ["COLORTERM", "LANG", "LANGUAGE", "LC_*", "LINGUAS", "TERM", "TZ"]

    env_delete                = ["IFS", "CDPATH", "LOCALDOMAIN", "RES_OPTIONS", "HOSTALIASES",
                                "NLSPATH", "PATH_LOCALE", "LD_*", "_RLD*", "TERMINFO", "TERMINFO_DIRS",
                                "TERMPATH", "TERMCAP", "ENV", "BASH_ENV", "PS4", "GLOBIGNORE",
                                "BASHOPTS", "SHELLOPTS", "JAVA_TOOL_OPTIONS", "PERLIO_DEBUG",
                                "PERLLIB", "PERL5LIB", "PERL5OPT", "PERL5DB", "FPATH", "NULLCMD",
                                "READNULLCMD", "ZDOTDIR", "TMPPREFIX", "PYTHONHOME", "PYTHONPATH",
                                "PYTHONINSPECT", "PYTHONUSERBASE", "RUBYLIB", "RUBYOPT", "*=()*"]
}

/// A custom parser to parse seconds as fractional "minutes", the format used by
/// passwd_timeout and timestamp_timeout.
fn fractional_minutes(input: &str) -> Option<i64> {
    if input.contains('.') {
        Some((input.parse::<f64>().ok()? * 60.0).floor() as i64)
    } else {
        Some(input.parse::<i64>().ok()? * 60)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[allow(clippy::bool_assert_comparison)]
    #[test]
    fn check() {
        let mut def = Settings::default();
        assert_eq! { def.always_query_group_plugin, false };
        assert_eq! { def.always_set_home, false };
        assert_eq! { def.env_reset, true };
        assert_eq! { def.mail_badpass, true };
        assert_eq! { def.match_group_by_gid, false };
        assert_eq! { def.use_pty, true };
        assert_eq! { def.visiblepw, false };
        assert_eq! { def.env_editor, true };
        assert_eq! { def.passwd_tries, 3 };
        assert_eq! { def.secure_path, None };
        assert_eq! { def.env_check, ["COLORTERM", "LANG", "LANGUAGE", "LC_*", "LINGUAS", "TERM", "TZ"].iter().map(|s| s.to_string()).collect() };
        assert_eq! { def.verifypw, enums::verifypw::all };

        negate("env_check").unwrap()(&mut def);
        negate("env_reset").unwrap()(&mut def);
        negate("secure_path").unwrap()(&mut def);
        negate("verifypw").unwrap()(&mut def);
        assert_eq! { def.always_query_group_plugin, false };
        assert_eq! { def.always_set_home, false };
        assert_eq! { def.env_reset, false };
        assert_eq! { def.mail_badpass, true };
        assert_eq! { def.match_group_by_gid, false };
        assert_eq! { def.use_pty, true };
        assert_eq! { def.visiblepw, false };
        assert_eq! { def.env_editor, true };
        assert_eq! { def.passwd_tries, 3 };
        assert_eq! { def.secure_path, None };
        assert! { def.env_check.is_empty() };
        assert_eq! { def.verifypw, enums::verifypw::never };

        let SettingKind::Flag(f) = set("env_reset").unwrap() else {
            panic!()
        };
        f(&mut def);
        let SettingKind::Text(f) = set("secure_path").unwrap() else {
            panic!()
        };
        f("/bin").unwrap()(&mut def);
        let SettingKind::Integer(f) = set("passwd_tries").unwrap() else {
            panic!()
        };
        f("5").unwrap()(&mut def);
        let SettingKind::Text(f) = set("verifypw").unwrap() else {
            panic!()
        };
        f("any").unwrap()(&mut def);
        assert_eq! { def.always_query_group_plugin, false };
        assert_eq! { def.always_set_home, false };
        assert_eq! { def.env_reset, true };
        assert_eq! { def.mail_badpass, true };
        assert_eq! { def.match_group_by_gid, false };
        assert_eq! { def.use_pty, true };
        assert_eq! { def.visiblepw, false };
        assert_eq! { def.env_editor, true };
        assert_eq! { def.passwd_tries, 5 };
        assert_eq! { def.secure_path, Some("/bin".into()) };
        assert! { def.env_check.is_empty() };
        assert_eq! { def.verifypw, enums::verifypw::any };

        assert!(set("notanoption").is_none());
        assert!(f("notanoption").is_none());
    }
}
