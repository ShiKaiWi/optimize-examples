pub struct ModeParser;

impl ModeParser {
    pub fn parse() -> Result<Option<String>, String> {
        let args: Vec<String> = std::env::args().collect();
        if args.len() == 1 {
            Ok(None)
        } else if args.len() == 3 && args[1] == "--mode" {
            Ok(Some(args[2].clone()))
        } else {
            Err("invalid usage, example: <bin_name> --mode <mode_name>".to_string())
        }
    }
}
