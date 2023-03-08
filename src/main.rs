use miner_lottery::unicorn;
use miner_lottery::utils::unicorn_selection::get_unicorn_prn;

fn main() {
    /*--- FOLLOWING IS A TEST ---*/

    // The input public keys for the UNiCORN
    let inputs = [
        "AAAAC3NzaC1lZDI1NTE5AAAAIISBNp/6cz4by6FhlAtSI5Dg3agtFlOjoPayidNEDd78".to_string(),
        "AAAAB3NzaC1yc2EAAAADAQABAAACAQDflRJbqp9Ru2f4oLeUjEjV7QxbtlM8DiuSmj6iWA7vv6Hb62cQeLRT3Un4yerjOOBrXd3s4psReCL4+oo3GmvOIRCPlpMqZZFPgHYyF8pGobwSZZHSKNPpIeNWM90hXenJ4zTym59W/+jU3dhe8AeaAZS0Qy09vsHr4K+7cAjsz1ebp0yKNK06Betsfis26tipf40QzWUwrn/UuUgdlpXG6H+bUNuZ2cWDVkuq4G00F7OCv3wEdtnAy8VKnpqVIWsjo7c1WWVPtlslcVv1gRbTNaZ9msyvaiQ+hUsJYo8VNmu9iONJGUa3PnkWMmy9Z4hIHPG/imtVrWr0UNCXPB1gahDUJrm22qOH0iwg7PB88X9W5ryihe7HN3Q1nVDpcLyUGoXessuFtbzugDkDkfiNkTz3AYRtikcL3F9gdpTZ0EtPuIXItplsdUi5Axng45HB3VwEcd9ehBMv0WmYzsF3pxyE5jQOscken91cdGFF0l6llhsXohZBkpvV2v+4XOM6NCsXATQVdNDpsrNIScczHKXT9J/aqO54BhrORiytPLBgJScEde65dYTbEIgvzxFJtNzHAveCN/A3L+C/TGC57lRRSsuG1bD/2S1Zy4XQHsbNWAdOaurO858ik13WC+Sn5frc81vMIZdqPU5/imgC9c2XYrcfSz82v9HnurO8nw==".to_string()
    ];

    // The fixed parameters for the UNiCORN
    let fixed_params = unicorn::UnicornFixedParam {
        modulus: "6864797660130609714981900799081393217269435300143305409394463459185543183397656052122559640661454554977296311391480858037121987999716643812574028291115057151".to_string(),
        iterations: 1000,
        security: 1,
    };

    let seed = unicorn::construct_seed(&inputs);
    let unicorn_info = unicorn::construct_unicorn(seed, &fixed_params);
    let prn = get_unicorn_prn(&unicorn_info, 0);
    let selection = prn as usize % inputs.len();

    println!("Selected: {}", inputs[selection]);
}
