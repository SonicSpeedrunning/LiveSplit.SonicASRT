#![no_std]
#![feature(type_alias_impl_trait, const_async_blocks)]
#![warn(
    clippy::complexity,
    clippy::correctness,
    clippy::perf,
    clippy::style,
    clippy::undocumented_unsafe_blocks,
    rust_2018_idioms
)]

use asr::{
    file_format::pe,
    future::{next_tick, retry},
    signature::Signature,
    time::Duration,
    timer::{self, TimerState},
    watcher::{Pair, Watcher},
    Address, Address32, Process,
};

asr::panic_handler!();
asr::async_main!(nightly);

async fn main() {
    let settings = Settings::register();

    loop {
        // Hook to the target process
        let process = Process::wait_attach(PROCESS_NAME).await;

        process
            .until_closes(async {
                // Once the target has been found and attached to, set up some default watchers
                let mut watchers = Watchers::default();

                // Perform memory scanning to look for the addresses we need
                let addresses = retry(|| Addresses::init(&process)).await;

                loop {
                    // Splitting logic. Adapted from OG LiveSplit:
                    // Order of execution
                    // 1. update() will always be run first. There are no conditions on the execution of this action.
                    // 2. If the timer is currently either running or paused, then the isLoading, gameTime, and reset actions will be run.
                    // 3. If reset does not return true, then the split action will be run.
                    // 4. If the timer is currently not running (and not paused), then the start action will be run.
                    update_loop(&process, &addresses, &mut watchers);

                    let timer_state = timer::state();
                    if timer_state == TimerState::Running || timer_state == TimerState::Paused {
                        if let Some(is_loading) = is_loading(&watchers, &settings) {
                            if is_loading {
                                timer::pause_game_time()
                            } else {
                                timer::resume_game_time()
                            }
                        }

                        if let Some(game_time) = game_time(&watchers, &settings, &addresses) {
                            timer::set_game_time(game_time)
                        }

                        if reset(&watchers, &settings) {
                            timer::reset()
                        } else if split(&watchers, &settings) {
                            timer::split()
                        }
                    }

                    if timer::state() == TimerState::NotRunning && start(&watchers, &settings) {
                        timer::start();
                        timer::pause_game_time();

                        if let Some(is_loading) = is_loading(&watchers, &settings) {
                            if is_loading {
                                timer::pause_game_time()
                            } else {
                                timer::resume_game_time()
                            }
                        }
                    }

                    next_tick().await;
                }
            })
            .await;
    }
}

#[derive(asr::user_settings::Settings)]
struct Settings {
    #[default = false]
    /// -------- START OPTIONS --------
    _start: bool,
    #[default = true]
    /// Enable auto start
    start: bool,
    #[default = false]
    /// -------- SPLIT OPTIONS: ALL-CUPS & GP MODE --------
    _split_single: bool,
    #[default = true]
    /// Ocean View
    ocean_view: bool,
    #[default = true]
    /// Samba Studios
    samba_studios: bool,
    #[default = true]
    /// Carrier Zone
    carrier_zone: bool,
    #[default = true]
    /// Dragon Canyon
    dragon_canyon: bool,
    #[default = true]
    /// Temple Trouble
    temple_trouble: bool,
    #[default = true]
    /// Galactic Parade
    galactic_parade: bool,
    #[default = true]
    /// Seasonal Shrines
    seasonal_shrines: bool,
    #[default = true]
    /// Rogue's Landing
    rogues_landing: bool,
    #[default = true]
    /// Dream Valley
    dream_valley: bool,
    #[default = true]
    /// Chilly Castle
    chilly_castle: bool,
    #[default = true]
    /// Graffiti City
    graffiti_city: bool,
    #[default = true]
    /// Sanctuary Falls
    sanctuary_falls: bool,
    #[default = true]
    /// Graveyard Gig
    graveyard_gig: bool,
    #[default = true]
    /// Adder's Lair
    adders_lair: bool,
    #[default = true]
    /// Burning Depths
    burning_depths: bool,
    #[default = true]
    /// Race of AGES
    race_of_ages: bool,
    #[default = true]
    /// Sunshine Tour
    sunshine_tour: bool,
    #[default = true]
    /// Shibuya Downtown
    shibuya_downtown: bool,
    #[default = true]
    /// Roulette Road
    roulette_road: bool,
    #[default = true]
    /// Egg Hangar
    egg_hangar: bool,
    #[default = true]
    /// Outrun Bay
    outrun_bay: bool,
    #[default = false]
    /// -------- SPLIT OPTIONS: WORLD TOUR --------
    _world_tour: bool,
    #[default = true]
    /// Coastal Cruise
    coastal_cruise: bool,
    #[default = true]
    /// Studio Scrapes
    studio_scrapes: bool,
    #[default = true]
    /// Battlezone Blast
    battlezone_blast: bool,
    #[default = true]
    /// Downtown Drift
    downtown_drift: bool,
    #[default = true]
    /// Monkey Mayhem
    monkey_mayhem: bool,
    #[default = true]
    /// Starry Speedway
    starry_speedway: bool,
    #[default = true]
    /// Roulette Rush
    roulette_rush: bool,
    #[default = true]
    /// Canyon Carnage
    canyon_carnage: bool,
    #[default = true]
    /// Snowball Shakedown
    snowball_shakedown: bool,
    #[default = true]
    /// Banana Boost
    banana_boost: bool,
    #[default = true]
    /// Shinobi Scramble
    shinobi_scramble: bool,
    #[default = true]
    /// Seaside Scrap
    seaside_scrap: bool,
    #[default = true]
    /// Tricky Traffic
    tricky_traffic: bool,
    #[default = true]
    /// Studio Scurry
    studio_scurry: bool,
    #[default = true]
    /// Graffiti Groove
    graffiti_groove: bool,
    #[default = true]
    /// Shaking Skies
    shaking_skies: bool,
    #[default = true]
    /// Neon Knockout
    neon_knockout: bool,
    #[default = true]
    /// Pirate Plunder
    pirate_plunder: bool,
    #[default = true]
    /// Adder Assault
    adder_assault: bool,
    #[default = true]
    /// Dreamy Drive
    dreamy_drive: bool,
    #[default = true]
    /// Sanctuary Speedway
    sanctuary_speedway: bool,
    #[default = true]
    /// Keil's Carnage
    keils_carnage: bool,
    #[default = true]
    /// Carrier Crisis
    carrier_crisis: bool,
    #[default = true]
    /// Sunshine Slide
    sunshine_slide: bool,
    #[default = true]
    /// Rogue Rings
    rogue_rings: bool,
    #[default = true]
    /// Seaside Skirmish
    seaside_skirmish: bool,
    #[default = true]
    /// Shrine Time
    shrine_time: bool,
    #[default = true]
    /// Hangar Hassle
    hangar_hassle: bool,
    #[default = true]
    /// Booty Boost
    booty_boost: bool,
    #[default = true]
    /// Racing Rangers
    racing_rangers: bool,
    #[default = true]
    /// Shinobi Showdown
    shinobi_showdown: bool,
    #[default = true]
    /// Ruin Run
    ruin_run: bool,
    #[default = true]
    /// Monkey Brawl
    monkey_brawl: bool,
    #[default = true]
    /// Crumbling Chaos
    crumbling_chaos: bool,
    #[default = true]
    /// Hatcher Hustle
    hatcher_hustle: bool,
    #[default = true]
    /// Death Egg Duel
    death_egg_duel: bool,
    #[default = true]
    /// Undertaker Overtaker
    undertaker_overtaker: bool,
    #[default = true]
    /// Golden Gauntlet
    golden_gauntlet: bool,
    #[default = true]
    /// Carnival Clash
    carnival_clash: bool,
    #[default = true]
    /// Curien Curves
    curien_curves: bool,
    #[default = true]
    /// Molten Mayhem
    molten_mayhem: bool,
    #[default = true]
    /// Speeding Seasons
    speeding_seasons: bool,
    #[default = true]
    /// Burning Boost
    burning_boost: bool,
    #[default = true]
    /// Ocean Outrun
    ocean_outrun: bool,
    #[default = true]
    /// Billy Backslide
    billy_backslide: bool,
    #[default = true]
    /// Carrier Charge
    carrier_charge: bool,
    #[default = true]
    /// Jet Set Jaunt
    jet_set_jaunt: bool,
    #[default = true]
    /// Arcade Annihilation
    arcade_annihilation: bool,
    #[default = true]
    /// Rapid Ruins
    rapid_ruins: bool,
    #[default = true]
    /// Zombie Zoom
    zombie_zoom: bool,
    #[default = true]
    /// Maracar Madness
    maracar_madness: bool,
    #[default = true]
    /// Nightmare Meander
    nightmare_meander: bool,
    #[default = true]
    /// Maraca Melee
    maraca_melee: bool,
    #[default = true]
    /// Castle Chaos
    castle_chaos: bool,
    #[default = true]
    /// Volcano Velocity
    volcano_velocity: bool,
    #[default = true]
    /// Ranger Rush
    ranger_rush: bool,
    #[default = true]
    /// Tokyo Takeover
    tokyo_takeover: bool,
    #[default = true]
    /// Fatal Finale
    fatal_finale: bool,
}

#[derive(Default)]
struct Watchers {
    run_start: Watcher<bool>,
    end_credits: Watcher<bool>,
    game_mode: Watcher<GameMode>,
    required_laps: Watcher<u8>,
    total_race_time: Watcher<Duration>,
    race_completed: Watcher<bool>,
    race_status: Watcher<u8>,
    igt: Watcher<Duration>,
    event_type: Watcher<u32>,
    track_id: Watcher<Tracks>,
    total_igt: Duration,
    progress_igt: Duration,
    coastal_cruise: Watcher<u8>,
    studio_scrapes: Watcher<u8>,
    battlezone_blast: Watcher<u8>,
    downtown_drift: Watcher<u8>,
    monkey_mayhem: Watcher<u8>,
    starry_speedway: Watcher<u8>,
    roulette_rush: Watcher<u8>,
    canyon_carnage: Watcher<u8>,
    snowball_shakedown: Watcher<u8>,
    banana_boost: Watcher<u8>,
    shinobi_scramble: Watcher<u8>,
    seaside_scrap: Watcher<u8>,
    tricky_traffic: Watcher<u8>,
    studio_scurry: Watcher<u8>,
    graffiti_groove: Watcher<u8>,
    shaking_skies: Watcher<u8>,
    neon_knockout: Watcher<u8>,
    pirate_plunder: Watcher<u8>,
    adder_assault: Watcher<u8>,
    dreamy_drive: Watcher<u8>,
    sanctuary_speedway: Watcher<u8>,
    keils_carnage: Watcher<u8>,
    carrier_crisis: Watcher<u8>,
    sunshine_slide: Watcher<u8>,
    rogue_rings: Watcher<u8>,
    seaside_skirmish: Watcher<u8>,
    shrine_time: Watcher<u8>,
    hangar_hassle: Watcher<u8>,
    booty_boost: Watcher<u8>,
    racing_rangers: Watcher<u8>,
    shinobi_showdown: Watcher<u8>,
    ruin_run: Watcher<u8>,
    monkey_brawl: Watcher<u8>,
    crumbling_chaos: Watcher<u8>,
    hatcher_hustle: Watcher<u8>,
    death_egg_duel: Watcher<u8>,
    undertaker_overtaker: Watcher<u8>,
    golden_gauntlet: Watcher<u8>,
    carnival_clash: Watcher<u8>,
    curien_curves: Watcher<u8>,
    molten_mayhem: Watcher<u8>,
    speeding_seasons: Watcher<u8>,
    burning_boost: Watcher<u8>,
    ocean_outrun: Watcher<u8>,
    billy_backslide: Watcher<u8>,
    carrier_charge: Watcher<u8>,
    jet_set_jaunt: Watcher<u8>,
    arcade_annihilation: Watcher<u8>,
    rapid_ruins: Watcher<u8>,
    zombie_zoom: Watcher<u8>,
    maracar_madness: Watcher<u8>,
    nightmare_meander: Watcher<u8>,
    maraca_melee: Watcher<u8>,
    castle_chaos: Watcher<u8>,
    volcano_velocity: Watcher<u8>,
    ranger_rush: Watcher<u8>,
    tokyo_takeover: Watcher<u8>,
    fatal_finale: Watcher<u8>,
}

struct Addresses {
    run_start: Address,
    run_start_2: Address,
    end_credits: Address,
    mode_select: Address,
    player_base: Address,
    race_completed: Address,
    race_status: Address,
    igt: Address,
    event_type: Address,
    sunshine_coast: Address,
}

impl Addresses {
    fn init(game: &Process) -> Option<Self> {
        let main_module_base = game.get_module_address(PROCESS_NAME).ok()?;
        let main_module_size = pe::read_size_of_image(game, main_module_base)? as _;
        let main_module = (main_module_base, main_module_size);

        // Check if the hooked process is 32-bit before continuing
        if pe::MachineType::read(game, main_module_base)? != pe::MachineType::X86 {
            return None;
        }

        let run_start = {
            const SIG: Signature<14> = Signature::new("80 3D ?? ?? ?? ?? 00 0F 85 ?? ?? ?? ?? 56");
            let ptr = SIG.scan_process_range(game, main_module)? + 2;
            game.read::<Address32>(ptr).ok()?.into()
        };

        let run_start_2 = {
            const SIG: Signature<11> = Signature::new("74 0E 83 3D ?? ?? ?? ?? 00 74 0E");
            let ptr = SIG.scan_process_range(game, main_module)? + 4;
            game.read::<Address32>(ptr).ok()?.into()
        };

        let end_credits = {
            const SIG: Signature<3> = Signature::new("7E 5C A1");
            let ptr = SIG.scan_process_range(game, main_module)? + 3;
            game.read::<Address32>(ptr).ok()?.into()
        };

        let mode_select = {
            const SIG: Signature<10> = Signature::new("A1 ?? ?? ?? ?? 83 F8 02 74 16");
            let ptr = SIG.scan_process_range(game, main_module)? + 1;
            game.read::<Address32>(ptr).ok()?.into()
        };

        let player_base = {
            const SIG: Signature<13> = Signature::new("A1 ?? ?? ?? ?? 85 C0 0F 84 8D 00 00 00");
            let ptr = SIG.scan_process_range(game, main_module)? + 1;
            game.read::<Address32>(ptr).ok()?.into()
        };

        let race_completed = {
            const SIG: Signature<11> = Signature::new("8B 04 24 A3 ?? ?? ?? ?? 83 C4 08");
            let ptr = SIG.scan_process_range(game, main_module)? + 4;
            game.read::<Address32>(ptr).ok()?.into()
        };

        let race_status = {
            const SIG: Signature<11> = Signature::new("7C 44 83 3D ?? ?? ?? ?? 00 74 3B");
            let ptr = SIG.scan_process_range(game, main_module)? + 4;
            game.read::<Address32>(ptr).ok()?.into()
        };

        let igt = {
            const SIG: Signature<7> = Signature::new("D8 05 ?? ?? ?? ?? 56");
            let ptr = SIG.scan_process_range(game, main_module)? + 2;
            game.read::<Address32>(ptr).ok()?.into()
        };

        let event_type = {
            const SIG: Signature<10> = Signature::new("55 8B E9 8B 0D ?? ?? ?? ?? 57");
            let ptr = SIG.scan_process_range(game, main_module)? + 5;
            game.read::<Address32>(ptr).ok()?.into()
        };

        let sunshine_coast = {
            const SIG: Signature<11> = Signature::new("8B 2C 85 ?? ?? ?? ?? 89 7C 24 20");
            let ptr = SIG.scan_process_range(game, main_module)? + 3;
            game.read::<Address32>(ptr).ok()?.into()
        };

        Some(Self {
            run_start,
            run_start_2,
            end_credits,
            mode_select,
            player_base,
            race_completed,
            race_status,
            igt,
            event_type,
            sunshine_coast,
        })
    }
}

fn update_loop(game: &Process, addresses: &Addresses, watchers: &mut Watchers) {
    watchers.run_start.update_infallible(
        game.read::<u8>(addresses.run_start)
            .is_ok_and(|value| value == 1)
            && game
                .read::<u8>(addresses.run_start_2)
                .is_ok_and(|value| value == 1),
    );

    watchers.end_credits.update_infallible(
        game.read_pointer_path32(addresses.end_credits, &[0, 0x8C])
            .unwrap_or_default(),
    );

    watchers
        .game_mode
        .update_infallible(match game.read::<u8>(addresses.mode_select) {
            Ok(0) => GameMode::WorldTour,
            Ok(1) => GameMode::GandPrix,
            Ok(2) => GameMode::TimeAttack,
            Ok(3) => GameMode::SingleRace,
            _ => {
                watchers
                    .game_mode
                    .pair
                    .unwrap_or_else(|| Pair {
                        current: GameMode::WorldTour,
                        old: GameMode::WorldTour,
                    })
                    .current
            }
        });

    let (required_laps, total_race_time) = {
        let mut required_laps: u8 = watchers.required_laps.pair.unwrap_or_default().current;
        let mut total_race_time = watchers.total_race_time.pair.unwrap_or_default().current;

        if let Ok(addr) = game.read::<Address32>(addresses.player_base) {
            if let Ok(addr) = game.read::<Address32>(addr + 0) {
                if let Ok(addr) = game.read::<Address32>(addr + 0xC1B8) {
                    if let Ok(r_l) = game.read(addr + 0x4) {
                        required_laps = r_l;
                    }
                    if let Ok(t_r_t) = game.read::<f32>(addr + 0x28) {
                        total_race_time = Duration::milliseconds((t_r_t * 100.0) as i64 * 10);
                    }
                }
            }
        }
        (required_laps, total_race_time)
    };
    watchers.required_laps.update_infallible(required_laps);
    watchers.total_race_time.update_infallible(total_race_time);

    watchers
        .race_completed
        .update_infallible(game.read(addresses.race_completed).unwrap_or_default());
    watchers
        .race_status
        .update_infallible(game.read(addresses.race_status).unwrap_or_default());
    watchers.igt.update_infallible({
        if let Ok(time) = game.read::<f32>(addresses.igt) {
            Duration::milliseconds((time * 100.0) as i64 * 10)
        } else {
            watchers.igt.pair.unwrap_or_default().current
        }
    });

    watchers.event_type.update_infallible(
        game.read_pointer_path32(addresses.event_type, &[0x0, 0x0])
            .unwrap_or_default(),
    );
    watchers.track_id.update_infallible(
        match game.read_pointer_path32::<u32>(addresses.event_type + 0x4, &[0x0, 0x0]) {
            Ok(0xD4257EBD) => Tracks::OceanView,
            Ok(0x32D305A8) => Tracks::SambaStudios,
            Ok(0xC72B3B98) => Tracks::CarrierZone,
            Ok(0x03EB7FFF) => Tracks::DragonCanyon,
            Ok(0xE3121777) => Tracks::TempleTrouble,
            Ok(0x4E015AB6) => Tracks::GalacticParade,
            Ok(0x503C1CBC) => Tracks::SeasonalShrines,
            Ok(0x7534B7CA) => Tracks::RoguesLanding,
            Ok(0x38A394ED) => Tracks::DreamValley,
            Ok(0xC5C9DEA1) => Tracks::ChillyCastle,
            Ok(0xD936550C) => Tracks::GraffitiCity,
            Ok(0x4A0FF7AE) => Tracks::SanctuaryFalls,
            Ok(0xCD8017BA) => Tracks::GraveyardGig,
            Ok(0xDC93F18B) => Tracks::AddersLair,
            Ok(0x2DB91FC2) => Tracks::BurningDepths,
            Ok(0x94610644) => Tracks::RaceOfAges,
            Ok(0xE6CD97F0) => Tracks::SushineTour,
            Ok(0xE87FDF22) => Tracks::ShibuyaDowntown,
            Ok(0x17463C8D) => Tracks::RouletteRoad,
            Ok(0xFEBC639E) => Tracks::EggHangar,
            Ok(0x1EF56CE1) => Tracks::OutrunBay,
            _ => {
                watchers
                    .track_id
                    .pair
                    .unwrap_or_else(|| Pair {
                        old: Tracks::OceanView,
                        current: Tracks::OceanView,
                    })
                    .current
            }
        },
    );

    let sunshine_coast = game
        .read::<Address32>(addresses.sunshine_coast)
        .unwrap_or_default();
    let mut stars = game
        .read::<[u8; 0x719]>(sunshine_coast)
        .unwrap_or_else(|_| [0; 0x719]);
    watchers.coastal_cruise.update_infallible(stars[0x7C]);
    watchers.studio_scrapes.update_infallible(stars[0x138]);
    watchers.battlezone_blast.update_infallible(stars[0x1F4]);
    watchers.downtown_drift.update_infallible(stars[0x2B0]);
    watchers.monkey_mayhem.update_infallible(stars[0x36C]);
    watchers.starry_speedway.update_infallible(stars[0x428]);
    watchers.roulette_rush.update_infallible(stars[0x4E4]);
    watchers.canyon_carnage.update_infallible(stars[0x5A0]);

    let frozen_valley = game
        .read::<Address32>(addresses.sunshine_coast + 0x4)
        .unwrap_or_default();
    stars = game
        .read::<[u8; 0x719]>(frozen_valley)
        .unwrap_or_else(|_| [0; 0x719]);
    watchers.snowball_shakedown.update_infallible(stars[0x7C]);
    watchers.banana_boost.update_infallible(stars[0x138]);
    watchers.shinobi_scramble.update_infallible(stars[0x1F4]);
    watchers.seaside_scrap.update_infallible(stars[0x2B0]);
    watchers.tricky_traffic.update_infallible(stars[0x36C]);
    watchers.studio_scurry.update_infallible(stars[0x428]);
    watchers.graffiti_groove.update_infallible(stars[0x4E4]);
    watchers.shaking_skies.update_infallible(stars[0x5A0]);
    watchers.neon_knockout.update_infallible(stars[0x65C]);
    watchers.pirate_plunder.update_infallible(stars[0x718]);

    let scorching_skies = game
        .read::<Address32>(addresses.sunshine_coast + 0x8)
        .unwrap_or_default();
    stars = game
        .read::<[u8; 0x719]>(scorching_skies)
        .unwrap_or_else(|_| [0; 0x719]);
    watchers.adder_assault.update_infallible(stars[0x7C]);
    watchers.dreamy_drive.update_infallible(stars[0x138]);
    watchers.sanctuary_speedway.update_infallible(stars[0x1F4]);
    watchers.keils_carnage.update_infallible(stars[0x2B0]);
    watchers.carrier_crisis.update_infallible(stars[0x36C]);
    watchers.sunshine_slide.update_infallible(stars[0x428]);
    watchers.rogue_rings.update_infallible(stars[0x4E4]);
    watchers.seaside_skirmish.update_infallible(stars[0x5A0]);
    watchers.shrine_time.update_infallible(stars[0x65C]);
    watchers.hangar_hassle.update_infallible(stars[0x718]);

    let twilight_engine = game
        .read::<Address32>(addresses.sunshine_coast + 0xC)
        .unwrap_or_default();
    stars = game
        .read::<[u8; 0x719]>(twilight_engine)
        .unwrap_or_else(|_| [0; 0x719]);
    watchers.booty_boost.update_infallible(stars[0x7C]);
    watchers.racing_rangers.update_infallible(stars[0x138]);
    watchers.shinobi_showdown.update_infallible(stars[0x1F4]);
    watchers.ruin_run.update_infallible(stars[0x2B0]);
    watchers.monkey_brawl.update_infallible(stars[0x36C]);
    watchers.crumbling_chaos.update_infallible(stars[0x428]);
    watchers.hatcher_hustle.update_infallible(stars[0x4E4]);
    watchers.death_egg_duel.update_infallible(stars[0x5A0]);
    watchers
        .undertaker_overtaker
        .update_infallible(stars[0x65C]);
    watchers.golden_gauntlet.update_infallible(stars[0x718]);

    let moonlight_park = game
        .read::<Address32>(addresses.sunshine_coast + 0x10)
        .unwrap_or_default();
    stars = game
        .read::<[u8; 0x719]>(moonlight_park)
        .unwrap_or_else(|_| [0; 0x719]);
    watchers.carnival_clash.update_infallible(stars[0x7C]);
    watchers.curien_curves.update_infallible(stars[0x138]);
    watchers.molten_mayhem.update_infallible(stars[0x1F4]);
    watchers.speeding_seasons.update_infallible(stars[0x2B0]);
    watchers.burning_boost.update_infallible(stars[0x36C]);
    watchers.ocean_outrun.update_infallible(stars[0x428]);
    watchers.billy_backslide.update_infallible(stars[0x4E4]);
    watchers.carrier_charge.update_infallible(stars[0x5A0]);
    watchers.jet_set_jaunt.update_infallible(stars[0x65C]);
    watchers.arcade_annihilation.update_infallible(stars[0x718]);

    let superstar_showdown = game
        .read::<Address32>(addresses.sunshine_coast + 0x14)
        .unwrap_or_default();
    stars = game
        .read::<[u8; 0x719]>(superstar_showdown)
        .unwrap_or_else(|_| [0; 0x719]);
    watchers.rapid_ruins.update_infallible(stars[0x7C]);
    watchers.zombie_zoom.update_infallible(stars[0x138]);
    watchers.maracar_madness.update_infallible(stars[0x1F4]);
    watchers.nightmare_meander.update_infallible(stars[0x2B0]);
    watchers.maraca_melee.update_infallible(stars[0x36C]);
    watchers.castle_chaos.update_infallible(stars[0x428]);
    watchers.volcano_velocity.update_infallible(stars[0x4E4]);
    watchers.ranger_rush.update_infallible(stars[0x5A0]);
    watchers.tokyo_takeover.update_infallible(stars[0x65C]);
    watchers.fatal_finale.update_infallible(stars[0x718]);

    if timer::state() == TimerState::NotRunning {
        watchers.total_igt = Duration::ZERO;
        watchers.progress_igt = Duration::ZERO;
    } else if let Some(race_completed) = &watchers.race_completed.pair {
        if let Some(igt) = &watchers.igt.pair {
            if !race_completed.current {
                if let Some(race_status) = &watchers.race_status.pair {
                    if igt.changed_to(&Duration::ZERO) && race_status.old == 4 {
                        watchers.total_igt += igt.old;
                        watchers.progress_igt = watchers.total_igt;
                    } else {
                        watchers.progress_igt = watchers.total_igt + igt.current;
                    }
                }
            } else if race_completed.changed_to(&true) {
                watchers.total_igt += {
                    if (watchers
                        .game_mode
                        .pair
                        .is_some_and(|gm| gm.current == GameMode::WorldTour)
                        && required_laps == 0xFF)
                        || watchers
                            .event_type
                            .pair
                            .is_some_and(|et| et.current == 0xE64B5DD8)
                    {
                        igt.current
                    } else {
                        total_race_time
                    }
                };
                watchers.progress_igt = watchers.total_igt;
            }
        }
    }
}

fn start(watchers: &Watchers, settings: &Settings) -> bool {
    if !settings.start {
        return false;
    }

    if !watchers
        .run_start
        .pair
        .is_some_and(|value| value.changed_to(&true))
    {
        return false;
    }

    match watchers.game_mode.pair {
        Some(x) => match x.current {
            GameMode::GandPrix | GameMode::SingleRace => true,
            GameMode::WorldTour => {
                watchers
                    .coastal_cruise
                    .pair
                    .is_some_and(|value| value.current == 0)
                    && watchers
                        .canyon_carnage
                        .pair
                        .is_some_and(|value| value.current == 0)
            }
            _ => false,
        },
        _ => false,
    }
}

fn split(watchers: &Watchers, settings: &Settings) -> bool {
    match watchers.game_mode.pair {
        Some(x) => match x.current {
            GameMode::WorldTour => {
                (watchers
                    .coastal_cruise
                    .pair
                    .is_some_and(|value| value.increased())
                    && settings.coastal_cruise)
                    || (watchers
                        .studio_scrapes
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.studio_scrapes)
                    || (watchers
                        .battlezone_blast
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.battlezone_blast)
                    || (watchers
                        .downtown_drift
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.downtown_drift)
                    || (watchers
                        .monkey_mayhem
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.monkey_mayhem)
                    || (watchers
                        .starry_speedway
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.starry_speedway)
                    || (watchers
                        .roulette_rush
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.roulette_rush)
                    || (watchers
                        .canyon_carnage
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.canyon_carnage)
                    || (watchers
                        .snowball_shakedown
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.snowball_shakedown)
                    || (watchers
                        .banana_boost
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.banana_boost)
                    || (watchers
                        .shinobi_scramble
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.shinobi_scramble)
                    || (watchers
                        .seaside_scrap
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.seaside_scrap)
                    || (watchers
                        .tricky_traffic
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.tricky_traffic)
                    || (watchers
                        .studio_scurry
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.studio_scurry)
                    || (watchers
                        .graffiti_groove
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.graffiti_groove)
                    || (watchers
                        .shaking_skies
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.shaking_skies)
                    || (watchers
                        .neon_knockout
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.neon_knockout)
                    || (watchers
                        .pirate_plunder
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.pirate_plunder)
                    || (watchers
                        .adder_assault
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.adder_assault)
                    || (watchers
                        .dreamy_drive
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.dreamy_drive)
                    || (watchers
                        .sanctuary_speedway
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.sanctuary_speedway)
                    || (watchers
                        .keils_carnage
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.keils_carnage)
                    || (watchers
                        .carrier_crisis
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.carrier_crisis)
                    || (watchers
                        .sunshine_slide
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.sunshine_slide)
                    || (watchers
                        .rogue_rings
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.rogue_rings)
                    || (watchers
                        .seaside_skirmish
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.seaside_skirmish)
                    || (watchers
                        .shrine_time
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.shrine_time)
                    || (watchers
                        .hangar_hassle
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.hangar_hassle)
                    || (watchers
                        .booty_boost
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.booty_boost)
                    || (watchers
                        .racing_rangers
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.racing_rangers)
                    || (watchers
                        .shinobi_showdown
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.shinobi_showdown)
                    || (watchers
                        .ruin_run
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.ruin_run)
                    || (watchers
                        .monkey_brawl
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.monkey_brawl)
                    || (watchers
                        .crumbling_chaos
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.crumbling_chaos)
                    || (watchers
                        .hatcher_hustle
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.hatcher_hustle)
                    || (watchers
                        .death_egg_duel
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.death_egg_duel)
                    || (watchers
                        .undertaker_overtaker
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.undertaker_overtaker)
                    || (watchers
                        .golden_gauntlet
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.golden_gauntlet)
                    || (watchers
                        .carnival_clash
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.carnival_clash)
                    || (watchers
                        .curien_curves
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.curien_curves)
                    || (watchers
                        .molten_mayhem
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.molten_mayhem)
                    || (watchers
                        .speeding_seasons
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.speeding_seasons)
                    || (watchers
                        .burning_boost
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.burning_boost)
                    || (watchers
                        .ocean_outrun
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.ocean_outrun)
                    || (watchers
                        .billy_backslide
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.billy_backslide)
                    || (watchers
                        .carrier_charge
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.carrier_charge)
                    || (watchers
                        .jet_set_jaunt
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.jet_set_jaunt)
                    || (watchers
                        .arcade_annihilation
                        .pair
                        .is_some_and(|value| value.changed_to(&4))
                        && settings.arcade_annihilation)
                    || (watchers
                        .end_credits
                        .pair
                        .is_some_and(|value| value.changed_to(&true))
                        && watchers
                            .arcade_annihilation
                            .pair
                            .is_some_and(|value| value.current != 4)
                        && settings.arcade_annihilation)
                    || (watchers
                        .rapid_ruins
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.rapid_ruins)
                    || (watchers
                        .zombie_zoom
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.zombie_zoom)
                    || (watchers
                        .maracar_madness
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.maracar_madness)
                    || (watchers
                        .nightmare_meander
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.nightmare_meander)
                    || (watchers
                        .maraca_melee
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.maraca_melee)
                    || (watchers
                        .castle_chaos
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.castle_chaos)
                    || (watchers
                        .volcano_velocity
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.volcano_velocity)
                    || (watchers
                        .ranger_rush
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.ranger_rush)
                    || (watchers
                        .tokyo_takeover
                        .pair
                        .is_some_and(|value| value.increased())
                        && settings.tokyo_takeover)
                    || (watchers
                        .fatal_finale
                        .pair
                        .is_some_and(|value| value.increased() && value.current != 4)
                        && settings.fatal_finale)
                    || (watchers
                        .end_credits
                        .pair
                        .is_some_and(|value| value.changed_to(&true))
                        && watchers
                            .fatal_finale
                            .pair
                            .is_some_and(|value| value.current == 4)
                        && settings.fatal_finale)
            }
            GameMode::GandPrix | GameMode::SingleRace => {
                watchers
                    .race_completed
                    .pair
                    .is_some_and(|value| value.changed_to(&true))
                    && match watchers.track_id.pair {
                        Some(x) => match x.current {
                            Tracks::OceanView => settings.ocean_view,
                            Tracks::SambaStudios => settings.samba_studios,
                            Tracks::CarrierZone => settings.carrier_zone,
                            Tracks::DragonCanyon => settings.dragon_canyon,
                            Tracks::TempleTrouble => settings.temple_trouble,
                            Tracks::GalacticParade => settings.galactic_parade,
                            Tracks::SeasonalShrines => settings.seasonal_shrines,
                            Tracks::RoguesLanding => settings.rogues_landing,
                            Tracks::DreamValley => settings.dream_valley,
                            Tracks::ChillyCastle => settings.chilly_castle,
                            Tracks::GraffitiCity => settings.graffiti_city,
                            Tracks::SanctuaryFalls => settings.sanctuary_falls,
                            Tracks::GraveyardGig => settings.graveyard_gig,
                            Tracks::AddersLair => settings.adders_lair,
                            Tracks::BurningDepths => settings.burning_depths,
                            Tracks::RaceOfAges => settings.race_of_ages,
                            Tracks::SushineTour => settings.sunshine_tour,
                            Tracks::ShibuyaDowntown => settings.shibuya_downtown,
                            Tracks::RouletteRoad => settings.roulette_road,
                            Tracks::EggHangar => settings.egg_hangar,
                            Tracks::OutrunBay => settings.outrun_bay,
                        },
                        _ => false,
                    }
            }
            _ => false,
        },
        _ => false,
    }
}

fn reset(_watchers: &Watchers, _settings: &Settings) -> bool {
    false
}

fn is_loading(_watchers: &Watchers, _settings: &Settings) -> Option<bool> {
    Some(true)
}

fn game_time(
    watchers: &Watchers,
    _settings: &Settings,
    _addresses: &Addresses,
) -> Option<Duration> {
    Some(watchers.progress_igt)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GameMode {
    WorldTour,
    GandPrix,
    TimeAttack,
    SingleRace,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tracks {
    OceanView,
    SambaStudios,
    CarrierZone,
    DragonCanyon,
    TempleTrouble,
    GalacticParade,
    SeasonalShrines,
    RoguesLanding,
    DreamValley,
    ChillyCastle,
    GraffitiCity,
    SanctuaryFalls,
    GraveyardGig,
    AddersLair,
    BurningDepths,
    RaceOfAges,
    SushineTour,
    ShibuyaDowntown,
    RouletteRoad,
    EggHangar,
    OutrunBay,
}

const PROCESS_NAME: &str = "ASN_App_PcDx9_Final.exe";
