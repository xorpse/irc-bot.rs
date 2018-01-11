use core::*;
use core::BotCmdAuthLvl as Auth;
use std::fs::File;
use yaml_rust::Yaml;

pub fn mk() -> Module {
    mk_module("test")
        .command(
            "test-line-wrap",
            "",
            "Request a long message from the bot, to test its line-wrapping function.",
            Auth::Admin,
            Box::new(test_line_wrap),
            &[],
        )
        .command(
            "test-error-handling",
            "",
            "This command's handler function returns an error, to test the bot framework's \
             error-handling mechanism(s).",
            Auth::Admin,
            Box::new(test_error_handling),
            &[],
        )
        .command(
            "test-panic-catching",
            "",
            "This command's handler function panics, to test the bot framework's panic-catching \
             mechanism.",
            Auth::Admin,
            Box::new(test_panic_catching),
            &[],
        )
        .command(
            "test-fs-sandbox",
            "",
            "Ask the bot whether it can read a file that it should be unable to read if the bot \
             framework's sandboxing mechanism is working.",
            Auth::Admin,
            Box::new(test_fs_sandbox),
            &[],
        )
        .end()
}

const LOREM_IPSUM_TEXT: &'static str =
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Integer et tincidunt nibh. Nullam \
     aliquet imperdiet cursus. Duis at turpis mollis, iaculis quam sed, efficitur arcu. Sed vel \
     massa sit amet magna efficitur hendrerit. Donec auctor auctor ligula nec semper. Nulla a \
     odio suscipit, suscipit velit in, ullamcorper velit. In bibendum pulvinar ipsum. Fusce \
     elementum maximus mattis. Donec sed mauris nec ante eleifend dapibus non faucibus massa. \
     Vivamus a auctor ligula. Cras hendrerit, velit sit amet sagittis placerat, elit elit feugiat \
     quam, vel aliquet ligula elit sit amet nibh. Fusce dignissim, orci vitae sodales ornare, \
     lacus risus facilisis sem, a imperdiet lectus massa at velit. Etiam sed magna congue, \
     pulvinar diam quis, facilisis risus. Sed semper, lectus vulputate luctus fermentum, quam \
     lacus consectetur arcu, ac mollis ipsum metus vel nunc. Ut posuere arcu enim, id dictum arcu \
     sagittis in. Mauris a lectus nec ligula eleifend rutrum. Class aptent taciti sociosqu ad \
     litora torquent per conubia massa nunc.";

fn test_line_wrap(_: &State, _: &MsgMetadata, _: &Yaml) -> BotCmdResult {
    BotCmdResult::Ok(Reaction::Reply(LOREM_IPSUM_TEXT.into()))
}

fn test_error_handling(_: &State, _: &MsgMetadata, _: &Yaml) -> BotCmdResult {
    BotCmdResult::BotErrMsg("An error for testing purposes.".into())
}

fn test_panic_catching(_: &State, _: &MsgMetadata, _: &Yaml) -> BotCmdResult {
    panic!("Panicking for testing purposes....")
}

fn test_fs_sandbox(_: &State, _: &MsgMetadata, _: &Yaml) -> Reaction {
    if !sandbox::PLATFORM_HAS_SANDBOX {
        return Reaction::Msg("Sandboxing not supported.".into());
    }

    let path = if cfg!(any(target_os = "linux", target_os = "macos")) {
        // I suspect there are better choices — especially, files that could better be depended
        // upon to be present — but this is the classic one.
        "/etc/passwd"
    } else if cfg!(target_os = "android") {
        unimplemented!("I don't know an appropriate path at which to look.")
    } else {
        unreachable!(
            "Apparently, `irc_bot::core::sandbox` and `irc_bot::modules::test` disagree on which \
             platforms have sandboxing support."
        )
    };

    match File::open(path) {
        Ok(_) => Reaction::Msg(
            "Alarmingly, I have succeeded in opening a file (for reading) to which I ought \
             have been denied access."
                .into(),
        ),
        Err(e) => Reaction::Msg(
            format!(
                "As hoped, I have failed to open a file (for reading) to which I ought be \
                 denied access, receiving the following error: {}",
                e
            ).into(),
        ),
    }
}
