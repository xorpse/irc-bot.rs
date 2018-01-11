use super::ModuleFeatureInfo;
use super::ModuleInfo;
use super::OutboxRecord;
use irc;
use serde_yaml;
use std::any::Any;
use std::borrow::Cow;
use std::io;
use std::sync::mpsc;
use util;

error_chain! {
    foreign_links {
        Io(io::Error);
        SerdeYaml(serde_yaml::Error);
    }

    links {
        IrcCrate(irc::error::Error, irc::error::ErrorKind);
        YamlUtil(util::yaml::Error, util::yaml::ErrorKind);
    }

    errors {
        IdentificationFailure(io_err: io::Error)

        ModuleRegistryClash(old: ModuleInfo, new: ModuleInfo)

        ModuleFeatureRegistryClash(old: ModuleFeatureInfo, new: ModuleFeatureInfo)

        Config(key: String, problem: String) {
            description("configuration error")
            display("Configuration error: Key {:?} {}.", key, problem)
        }

        HandlerPanic(
            feature_kind: Cow<'static, str>,
            feature_name: Cow<'static, str>,
            payload: Box<Any + Send + 'static>
        ) {
            description("panic in module feature handler function")
            display("The handler function for {} {:?} panicked with the following message: {}",
                    feature_kind,
                    feature_name,
                    util::fmt::FmtAny(payload.as_ref()))
        }

        MsgPrefixUpdateRequestedButPrefixMissing

        NicknameUnknown {
            description("nickname retrieval error")
            display("Puzzlingly, the bot seems to have forgotten its own nickname.")
        }

        LockPoisoned(lock_contents_desc: Cow<'static, str>) {
            description("lock poisoned")
            display("A thread panicked, poisoning a lock around {}.", lock_contents_desc)
        }

        SandboxFailed {
            description("sandboxing failed")
            display("The bot framework failed to sandbox the process, even though the framework \
                     should support sandboxing on this platform.")
        }

        SandboxUnsupported {
            description("sandboxing unsupported")
            display("The bot framework could not sandbox the process, because the framework does \
                     not support sandboxing on this platform.")
        }

        Unit {
            description("unknown error")
            display("An error seems to have occurred, but unfortunately the error type provided \
                     was the unit type, containing no information about the error.")
        }
    }
}
