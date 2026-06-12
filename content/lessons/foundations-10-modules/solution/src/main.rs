mod parser;

use parser::parse_data;
use parser::utils::clean;

fn main() {
    clean();
    parse_data();
}
