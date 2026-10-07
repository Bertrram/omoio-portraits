//! Which figure each of the game's portraits shows. The game names its
//! pictures by its own code names ("CatGryphon" is Scratch, "Spectrum" is
//! Flashwing, "2012" marks a Series 2 figure), with a few misspelt, so the
//! table was made by matching them to Cemu's figure list by name and
//! checking the rest by eye. A figure is the id and variant on the toy, as
//! Cemu's figure maker lists them; a variant without its own picture uses
//! its figure's plain one, which Omoio looks for itself.

/// A whole figure's portrait, 256 x 256: the picture's name in the game,
/// then the figure's id and variant.
pub const PORTRAITS: &[(&str, u16, u16)] = &[
    ("Whirlwind_WiiPortrait", 0, 0x0000), // Whirlwind
    ("Whirlwind2012_WiiPortrait", 0, 0x1801), // Series 2 Whirlwind
    ("PolarWhirlwind_WiiPortrait", 0, 0x1c02), // Polar Whirlwind
    ("HornBlastWhirlwind_WiiPortrait", 0, 0x2805), // Horn Blast Whirlwind
    ("SonicBoom_WiiPortrait", 1, 0x0000), // Sonic Boom
    ("SonicBoom2012_WiiPortrait", 1, 0x1801), // Series 2 Sonic Boom
    ("Warnado_WiiPortrait", 2, 0x0000), // Warnado
    ("LightCoreWarnado_WiiPortrait", 2, 0x2206), // LightCore Warnado
    ("LightningRod_WiiPortrait", 3, 0x0000), // Lightning Rod
    ("LightningRod2012_WiiPortrait", 3, 0x1801), // Series 2 Lightning Rod
    ("Bash_WiiPortrait", 4, 0x0000), // Bash
    ("Bash2012_WiiPortrait", 4, 0x1801), // Series 2 Bash
    ("Terrafin_WiiPortrait", 5, 0x0000), // Terrafin
    ("Terrafin2012_WiiPortrait", 5, 0x1801), // Series 2 Terrafin
    ("KnockoutTerrafin_WiiPortrait", 5, 0x2805), // Knockout Terrafin
    ("DinoRang_WiiPortrait", 6, 0x0000), // Dino Rang
    ("PrismBreak_WiiPortrait", 7, 0x0000), // Prism Break
    ("LightCorePrismBreak_WiiPortrait", 7, 0x1206), // LightCore Prism Break
    ("PrismBreak2012_WiiPortrait", 7, 0x1801), // Series 2 Prism Break
    ("HyperBeamPrismBreak_WiiPortrait", 7, 0x2805), // Hyper Beam Prism Break
    ("Sunburn_WiiPortrait", 8, 0x0000), // Sunburn
    ("Eruptor_WiiPortrait", 9, 0x0000), // Eruptor
    ("LightCoreEruptor_WiiPortrait", 9, 0x1206), // LightCore Eruptor
    ("Eruptor2012_WiiPortrait", 9, 0x1801), // Series 2 Eruptor
    ("Eruptor2013_WiiPortrait", 9, 0x2805), // Lava Barf Eruptor
    ("LavaEruptor__WiiPortrait", 9, 0x2c02), // Volcanic Eruptor
    ("Ignitor_WiiPortrait", 10, 0x0000), // Ignitor
    ("Ignitor2012_WiiPortrait", 10, 0x1801), // Series 2 Ignitor
    ("LegendaryIgnitor_WiiPortrait", 10, 0x1c03), // Legendary Ignitor
    ("Flameslinger_WiiPortrait", 11, 0x0000), // Flameslinger
    ("Flameslinger2012_WiiPortrait", 11, 0x1801), // Series 2 Flameslinger
    ("Zap_WiiPortrait", 12, 0x0000), // Zap
    ("Zap2012_WiiPortrait", 12, 0x1801), // Series 2 Zap
    ("Whamshell_WiiPortrait", 13, 0x0000), // Wham Shell
    ("LightCoreWhamshell_WiiPortrait", 13, 0x2206), // LightCore Wham Shell
    ("GillGrunt_WiiPortrait", 14, 0x0000), // Gill Grunt
    ("Gillgrunt2012_WiiPortrait", 14, 0x1801), // Series 2 Gill Grunt
    ("AnchorsGillGrunt_WiiPortrait", 14, 0x2805), // Anchors Away Gill Grunt
    ("SlamBam_WiiPortrait", 15, 0x0000), // Slam Bam
    ("SlamBam2012_WiiPortrait", 15, 0x1801), // Series 2 Slam Bam
    ("LegendarySlamBam_WiiPortrait", 15, 0x1c03), // Legendary Slam Bam
    ("Spyro_WiiPortrait", 16, 0x0000), // Spyro
    ("Spyro2012_WiiPortrait", 16, 0x1801), // Series 2 Spyro
    ("MegaRamSpyro_WiiPortrait", 16, 0x2805), // Mega Ram Spyro
    ("MegaRamDarkSpyro_WiiPortrait", 16, 0x2c02), // Dark Mega Ram Spyro
    ("Voodood_WiiPortrait", 17, 0x0000), // Voodood
    ("DoubleTrouble_WiiPortrait", 18, 0x0000), // Double Trouble
    ("DoubleTrouble2012_WiiPortrait", 18, 0x1801), // Series 2 Double Trouble
    ("RoyalDouble_WiiPortrait", 18, 0x1c02), // Royal Double Trouble
    ("TriggerHappy_WiiPortrait", 19, 0x0000), // Trigger Happy
    ("TriggerHappy2012_WiiPortrait", 19, 0x1801), // Series 2 Trigger Happy
    ("BigBangTriggerHappy_WiiPortrait", 19, 0x2805), // Big Bang Trigger Happy
    ("EasterTriggerHappy_WiiPortrait", 19, 0x2c02), // Springtime Trigger Happy
    ("Drobot_WiiPortrait", 20, 0x0000), // Drobot
    ("LightCoreDrobot_WiiPortrait", 20, 0x1206), // LightCore Drobot
    ("Drobot2012_WiiPortrait", 20, 0x1801), // Series 2 Drobot
    ("DrillSergent_WiiPortrait", 21, 0x0000), // Drill Sergeant
    ("DrillSergeant2012_WiiPortrait", 21, 0x1801), // Series 2 Drill Sergeant
    ("Boomer_WiiPortrait", 22, 0x0000), // Boomer
    ("Wreckingball_WiiPortrait", 23, 0x0000), // Wrecking Ball
    ("WreckingBall2012_WiiPortrait", 23, 0x1801), // Series 2 Wrecking Ball
    ("Camo_WiiPortrait", 24, 0x0000), // Camo
    ("ThornHornCamo_WiiPortrait", 24, 0x2805), // Thorn Horn Camo
    ("Zook_WiiPortrait", 25, 0x0000), // Zook
    ("Zook2012_WiiPortrait", 25, 0x1801), // Series 2 Zook
    ("StealthElf_WiiPortrait", 26, 0x0000), // Stealth Elf
    ("StealthElf2012_WIiPortrait", 26, 0x1801), // Series 2 Stealth Elf
    ("LegendaryStealthElf_WiiPortrait", 26, 0x1c03), // Legendary Stealth Elf
    ("NinjaStealthElf_WiiPortrait", 26, 0x2805), // Ninja Stealth Elf
    ("DarkNinjaStealthElf_WiiPortrait", 26, 0x2c02), // Dark Stealth Elf
    ("StumpSmash_WiiPortrait", 27, 0x0000), // Stump Smash
    ("StumpSmash2012_WiiPortrait", 27, 0x1801), // Series 2 Stump Smash
    ("DarkSpyro_WiiPortrait", 28, 0x0000), // Dark Spyro
    ("Hex_WiiPortrait", 29, 0x0000), // Hex
    ("LightCoreHex_WiiPortrait", 29, 0x1206), // LightCore Hex
    ("Hex2012_WiiPortrait", 29, 0x1801), // Series 2 Hex
    ("ChopChop_WiiPortrait", 30, 0x0000), // Chop Chop
    ("ChopChop2012_WiiPortrait", 30, 0x1801), // Series 2 Chop Chop
    ("TwinBladeChopChop_WiiPortrait", 30, 0x2805), // Twin Blade Chop Chop
    ("GhostRoaster_WiiPortrait", 31, 0x0000), // Ghost Roaster
    ("Cynder_WiiPortrait", 32, 0x0000), // Cynder
    ("Cynder2012_WiiPortrait", 32, 0x1801), // Series 2 Cynder
    ("PhantomCynder_WiiPortrait", 32, 0x2805), // Phantom Cynder
    ("JetVac_WiiPortrait", 100, 0x0000), // Jet Vac
    ("LightCoreJetVac_WiiPortrait", 100, 0x1206), // LightCore Jet Vac
    ("LegendaryJetVac_WiiPortrait", 100, 0x1403), // Legendary Jet Vac
    ("TurboJetVac_WiiPortrait", 100, 0x2805), // Turbo Jet Vac
    ("Swarm_WiiPortrait", 101, 0x0000), // Swarm
    ("Crusher_WiiPortrait", 102, 0x0000), // Crusher
    ("GraniteCrusher_WiiPortrait", 102, 0x1602), // Granite Crusher
    ("Spectrum_WiiPortrait", 103, 0x0000), // Flashwing
    ("JadeSpectrum_WiiPortrait", 103, 0x1402), // Jade Flashwing
    ("LightCoreSpectrum_WiiPortrait", 103, 0x2206), // LightCore Flashwing
    ("HotHead_WiiPortrait", 104, 0x0000), // Hot Head
    ("HotDog_WiiPortrait", 105, 0x0000), // Hot Dog
    ("MoltenHotDog_WiiPortrait", 105, 0x1402), // Molten Hot Dog
    ("FireBoneHotDog_WiiPortrait", 105, 0x2805), // Fire Bone Hot Dog
    ("Chill_WiiPortrait", 106, 0x0000), // Chill
    ("LightCoreChill_WiiPortrait", 106, 0x1206), // LightCore Chill
    ("LegendaryChill_WiiPortrait", 106, 0x1603), // Legendary Chill
    ("BlizzardChill_WiiPortrait", 106, 0x2805), // Blizzard Chill
    ("Thumpback_WiiPortrait", 107, 0x0000), // Thumpback
    ("PopFizz_WiiPortrait", 108, 0x0000), // Pop Fizz
    ("LightCorePopFizz_WiiPortrait", 108, 0x1206), // LightCore Pop Fizz
    ("PunchPopFizz_WiiPortrait", 108, 0x1402), // Punch Pop Fizz
    ("SuperGulpPopFizz_WiiPortrait", 108, 0x2805), // Super Gulp Pop Fizz
    ("Ninjini_WiiPortrait", 109, 0x0000), // Ninjini
    ("ScarletNinjini_WiiPortrait", 109, 0x1602), // Scarlet Ninjini
    ("Bouncer_WiiPortrait", 110, 0x0000), // Bouncer
    ("LegendaryBounce_WiiPortrait", 110, 0x1603), // Legendary Bouncer
    ("Sprocket_WiiPortrait", 111, 0x0000), // Sprocket
    ("HeavySprocket_WiiPortrait", 111, 0x2805), // Heavy Duty Sprocket
    ("TreeRex_WiiPortrait", 112, 0x0000), // Tree Rex
    ("GnarlyTreeRex_WiiPortrait", 112, 0x1602), // Gnarly Tree Rex
    ("ShroomBoom_WiiPortrait", 113, 0x0000), // Shroomboom
    ("LightCoreShroomBoom_WiiPortrait", 113, 0x1206), // LightCore Shroomboom
    ("EyeBrawl_WiiPortrait", 114, 0x0000), // Eye Brawl
    ("FrightRider_WiiPortrait", 115, 0x0000), // Fright Rider
    ("LegendaryBash_WiiPortrait", 404, 0x0000), // Legendary Bash
    ("LegendarySpryo_WiiPortrait", 416, 0x0000), // Legendary Spyro
    ("LegendaryTriggerHappy_WiiPortrait", 419, 0x0000), // Legendary Trigger Happy
    ("LegendaryChopChop_WiiPortrait", 430, 0x0000), // Legendary Chop Chop
    ("CatGryphon_WiiPortrait", 3000, 0x0000), // Scratch
    ("Puffer_WiiPortrait", 3001, 0x0000), // Pop Thorn
    ("Lockjaw_WiiPortrait", 3002, 0x0000), // Slobber Tooth
    ("DarkLockJaw_WiiPortrait", 3002, 0x2402), // Dark Slobber Tooth
    ("Scorpion_WiiPortrait", 3003, 0x0000), // Scorp
    ("Fryno_WiiPortrait", 3004, 0x0000), // Fryno
    ("Eclipse_WiiPortrait", 3005, 0x0000), // Smolderdash
    ("LightCoreEclipse_WiiPortrait", 3005, 0x2206), // LightCore Smolderdash
    ("Beetree_WiiPortrait", 3006, 0x0000), // Bumble Blast
    ("LightcoreBeeTree_WiiPortrait", 3006, 0x2206), // LightCore Bumble Blast
    ("HolidayBeeTree_WiiPortrait", 3006, 0x2402), // Jolly Bumble Blast
    ("Shaman_WiiPortrait", 3007, 0x0000), // Zoo Lou
    ("LegendaryShaman_WiiPortrait", 3007, 0x2403), // Legendary Zoo Lou
    ("Dunebug_WiiPortrait", 3008, 0x0000), // Dune Bug
    ("Fangirl_WiiPortrait", 3009, 0x0000), // Star Strike
    ("LightCoreFanGirl_WiiPortrait", 3009, 0x2206), // LightCore Star Strike
    ("LightCoreEnchantedFanGirl_WiiPortrait", 3009, 0x2602), // Enchanted Star Strike
    ("DaBomb_WiiPortrait", 3010, 0x0000), // Countdown
    ("LightCoreDaBomb_WiiPortrait", 3010, 0x2206), // LightCore Countdown
    ("SoccerDaBomb_WiiPortrait", 3010, 0x2402), // Kickoff Countdown
    ("GearBox_WiiPortrait", 3011, 0x0000), // Wind Up
    ("VVWindup_WiiPortrait", 3011, 0x2404), // Gear Head Wind Up
    ("Topsy_WiiPortrait", 3012, 0x0000), // Roller Brawl
    ("Grimm_WiiPortrait", 3013, 0x0000), // Grim Creeper
    ("LightCoreGrimm_WiiPortrait", 3013, 0x2206), // LightCore Grim Creeper
    ("LegendaryGrimm_WiiPortrait", 3013, 0x2603), // Legendary Grim Creeper
    ("Swordfish_WiiPortrait", 3014, 0x0000), // Rip Tide
    ("EelGirl_WiiPortrait", 3015, 0x0000), // Punk Shock
];

/// The swappers, by the name the game gives each one's halves, in the order
/// of their ids: bottoms from 1000, tops from 2000. A top is drawn as
/// `Body_<name>_Combo` and a bottom as `Legs_<name>_Combo`, both 512 x 512
/// on one canvas, so any top laid over any bottom makes the whole figure.
pub const SWAPPERS: [&str; 16] = [
    "Aviator",     // Boom Jet
    "Stormbird",   // Free Ranger
    "Miner",       // Rubble Rouser
    "Statue",      // Doom Stone
    "Fireknight",  // Blast Zone
    "Fireworks",   // Fire Kraken
    "Ninja",       // Stink Bomb
    "Gorilla",     // Grilla Drilla
    "Fortune",     // Hoot Loop
    "Panthers",    // Trap Shadow
    "Magnet",      // Magna Charge
    "Spyder",      // Spy Rise
    "Vampire",     // Night Shift
    "Cowboy",      // Rattle Shake
    "Freeze",      // Freeze Blade
    "Pirate",      // Wash Buckler
];

pub const TOPS: u16 = 2000;
pub const BOTTOMS: u16 = 1000;

/// The figure a picture of the game's shows, by the picture's name.
pub fn figure(source: &str) -> Option<(u16, u16)> {
    if let Some(&(_, id, variant)) = PORTRAITS.iter().find(|(name, _, _)| *name == source) {
        return Some((id, variant));
    }
    let swapper = |prefix: &str| {
        let name = source.strip_prefix(prefix)?.strip_suffix("_Combo")?;
        SWAPPERS.iter().position(|&known| known == name).map(|at| at as u16)
    };
    swapper("Body_")
        .map(|at| (TOPS + at, 0))
        .or_else(|| swapper("Legs_").map(|at| (BOTTOMS + at, 0)))
}

/// The file Omoio wants for one of Trap Team's pictures, from the name the
/// game's archive gives it. Figures and traps are named by id and variant,
/// the variant in decimal (`.../skylander_toys/2014/<group>/462_12288_snapshot.png/<hash>.igz`,
/// `.../traps/square/217_12291.png/...`), and become `<id>-<variant>` as for
/// SWAP Force. A villain is numbered (`.../villains/captured/1001_villaincaptured.png/...`)
/// and becomes `villain-<number>`, or `villain-<number>-loose` for the
/// picture the game shows of one out of a trap.
pub fn trap_team(path: &str) -> Option<String> {
    let mut parts = path.rsplit('/');
    parts.next()?;
    let picture = parts.next()?.strip_suffix(".png")?;
    let folder = parts.next()?;
    let number = |ending: &str| picture.strip_suffix(ending)?.parse::<u16>().ok();
    match folder {
        "captured" => number("_villaincaptured").map(|n| format!("villain-{n}")),
        "escaped" => number("_villainescaped").map(|n| format!("villain-{n}-loose")),
        _ if path.contains("/skylander_toys/") || path.contains("/traps/square/") => {
            let mut fields = picture.split('_');
            let id: u16 = fields.next()?.parse().ok()?;
            let variant: u16 = fields.next()?.parse().ok()?;
            Some(format!("{id}-{variant:04x}"))
        }
        _ => None,
    }
}

/// Skylanders Giants' Collection screen, which shows each figure it knows as
/// a round close-up, 256 x 256: the picture's name, then the figure's id and
/// variant. The names are code names too ("eagle" is Jet Vac), so the table
/// was checked by eye; the screen's own name for each says its id. The
/// variants are those Cemu's and RPCS3's lists give, but every sidekick is
/// given as variant 0, the plain figure Omoio falls back to, as the lists
/// give some of them only in later versions.
pub const GIANTS: &[(&str, u16, u16)] = &[
    ("airdragon", 0, 0x0000), // Whirlwind
    ("airdragon_series2", 0, 0x1801), // Series 2 Whirlwind
    ("airdragon_series2_alt", 0, 0x1c02), // Polar Whirlwind
    ("griffin", 1, 0x0000), // Sonic Boom
    ("griffin_series2", 1, 0x1801), // Series 2 Sonic Boom
    ("waterdragon", 2, 0x0000), // Warnado
    ("stormgiant", 3, 0x0000), // Lightning Rod
    ("stormgiant_series2", 3, 0x1801), // Series 2 Lightning Rod
    ("rockdragon", 4, 0x0000), // Bash
    ("rockdragon_series2", 4, 0x1801), // Series 2 Bash
    ("landshark", 5, 0x0000), // Terrafin
    ("landshark_series2", 5, 0x1801), // Series 2 Terrafin
    ("dinorang", 6, 0x0000), // Dino Rang
    ("gemgolem", 7, 0x0000), // Prism Break
    ("gemgolem_lightcore", 7, 0x1206), // LightCore Prism Break
    ("gemgolem_series2", 7, 0x1801), // Series 2 Prism Break
    ("phoenixdragon", 8, 0x0000), // Sunburn
    ("eruptor", 9, 0x0000), // Eruptor
    ("eruptor_lightcore", 9, 0x1206), // LightCore Eruptor
    ("eruptor_series2", 9, 0x1801), // Series 2 Eruptor
    ("flameknight", 10, 0x0000), // Ignitor
    ("flameknight_series2", 10, 0x1801), // Series 2 Ignitor
    ("flameknight_series2legendary", 10, 0x1c03), // Legendary Ignitor
    ("firearcher", 11, 0x0000), // Flameslinger
    ("firearcher_series2", 11, 0x1801), // Series 2 Flameslinger
    ("waterdragon2", 12, 0x0000), // Zap
    ("waterdragon2_series2", 12, 0x1801), // Series 2 Zap
    ("crustbuckler", 13, 0x0000), // Wham Shell
    ("gillgrunt", 14, 0x0000), // Gill Grunt
    ("gilgrunt_series2", 14, 0x1801), // Series 2 Gill Grunt
    ("yeti", 15, 0x0000), // Slam Bam
    ("yeti_series2", 15, 0x1801), // Series 2 Slam Bam
    ("yetilegendary", 15, 0x1c03), // Legendary Slam Bam
    ("spyrojr", 16, 0x0000), // Spyro
    ("spyro_series2", 16, 0x1801), // Series 2 Spyro
    ("skullorc", 17, 0x0000), // Voodood
    ("tikiwizard", 18, 0x0000), // Double Trouble
    ("tikiwizard_series2", 18, 0x1801), // Series 2 Double Trouble
    ("tikiwizard_series2_alt", 18, 0x1c02), // Royal Double Trouble
    ("goldencheat", 19, 0x0000), // Trigger Happy
    ("goldencheat_series2", 19, 0x1801), // Series 2 Trigger Happy
    ("metaldragon", 20, 0x0000), // Drobot
    ("metaldragon_lightcore", 20, 0x1206), // LightCore Drobot
    ("metaldragon_series2", 20, 0x1801), // Series 2 Drobot
    ("drillbot", 21, 0x0000), // Drill Sergeant
    ("drillbot_series2", 21, 0x1801), // Series 2 Drill Sergeant
    ("troll", 22, 0x0000), // Boomer
    ("balldragon", 23, 0x0000), // Wrecking Ball
    ("balldragon_series2", 23, 0x1801), // Series 2 Wrecking Ball
    ("plantdragon", 24, 0x0000), // Camo
    ("bambazooker", 25, 0x0000), // Zook
    ("bambazooker_series2", 25, 0x1801), // Series 2 Zook
    ("stealthelf", 26, 0x0000), // Stealth Elf
    ("stealthelf_series2", 26, 0x1801), // Series 2 Stealth Elf
    ("stealthelflegendary", 26, 0x1c03), // Legendary Stealth Elf
    ("stumpsmashent", 27, 0x0000), // Stump Smash
    ("stumpsmash_series2", 27, 0x1801), // Series 2 Stump Smash
    ("darkspyro", 28, 0x0000), // Dark Spyro
    ("shadowmaid", 29, 0x0000), // Hex
    ("shadowmaidlightcore", 29, 0x1206), // LightCore Hex
    ("shadowmaid_series2", 29, 0x1801), // Series 2 Hex
    ("pandoranguard", 30, 0x0000), // Chop Chop
    ("pandoranguard_series2", 30, 0x1801), // Series 2 Chop Chop
    ("ghosteater", 31, 0x0000), // Ghost Roaster
    ("cynder", 32, 0x0000), // Cynder
    ("cynder_series2", 32, 0x1801), // Series 2 Cynder
    ("eagle", 100, 0x0000), // Jet Vac
    ("eaglelightcore", 100, 0x1206), // LightCore Jet Vac
    ("eaglelegendary", 100, 0x1403), // Legendary Jet Vac
    ("giantair", 101, 0x0000), // Swarm
    ("earthgiant", 102, 0x0000), // Crusher
    ("earthgiant_alt", 102, 0x1602), // Granite Crusher
    ("earthdragon", 103, 0x0000), // Flashwing
    ("earthdragon_alt", 103, 0x1402), // Jade Flashwing
    ("firegiant", 104, 0x0000), // Hot Head
    ("firedog", 105, 0x0000), // Hot Dog
    ("firedog_alt", 105, 0x1402), // Molten Hot Dog
    ("icevalkyrie", 106, 0x0000), // Chill
    ("icevalkyrielightcore", 106, 0x1206), // LightCore Chill
    ("icevalkyrielegendarylightcore", 106, 0x1603), // Legendary Chill
    ("whale", 107, 0x0000), // Thumpback
    ("alchemist", 108, 0x0000), // Pop Fizz
    ("alchemistlightcore", 108, 0x1206), // LightCore Pop Fizz
    ("alchemistred", 108, 0x1402), // Punch Pop Fizz
    ("giantmagic", 109, 0x0000), // Ninjini
    ("giantmagic_alt", 109, 0x1602), // Scarlet Ninjini
    ("robogiant2", 110, 0x0000), // Bouncer
    ("robogiant2legendary", 110, 0x1603), // Legendary Bouncer
    ("steampunkgirl", 111, 0x0000), // Sprocket
    ("titanlife", 112, 0x0000), // Tree Rex
    ("titanlife_alt", 112, 0x1602), // Gnarly Tree Rex
    ("shroomy", 113, 0x0000), // Shroomboom
    ("shroomylightcore", 113, 0x1206), // LightCore Shroomboom
    ("cyclopsgiant", 114, 0x0000), // Eye Brawl
    ("undeadrider", 115, 0x0000), // Fright Rider
    ("magicitem_anvilrain", 200, 0x0000), // Anvil Rain
    ("magicitem_treasurechest", 201, 0x0000), // Hidden Treasure
    ("magicitem_healingelixir", 202, 0x0000), // Healing Elixir
    ("magicitem_ghostswords", 203, 0x0000), // Ghost Pirate Swords
    ("magicitems_timetwister", 204, 0x0000), // Time Twist Hourglass
    ("magicitem_skyironshield", 205, 0x0000), // Sky Iron Shield
    ("magicitem_wingedboots", 206, 0x0000), // Winged Boots
    ("magicitem_sparx", 207, 0x0000), // Sparx the Dragonfly
    ("magicitem_cannon", 208, 0x1206), // Dragonfire Cannon
    ("magicitem_cannonlegendary", 208, 0x1602), // the gold Dragonfire Cannon, with the variant Trap Team gives it
    ("magicitem_catapult", 209, 0x1206), // Scorpion Striker
    ("magicitem_dragonspeak", 300, 0x0000), // Dragon's Peak
    ("magicitem_empireofice", 301, 0x0000), // Empire of Ice
    ("magicitem_pirateship", 302, 0x0000), // Pirate Seas
    ("magicitem_darklightcrypt", 303, 0x0000), // Darklight Crypt
    ("magicitem_volcanicvault", 304, 0x0000), // Volcanic Vault
    ("rockdragonlegendary", 404, 0x0000), // Legendary Bash
    ("spyrojrlegendary", 416, 0x0000), // Legendary Spyro
    ("goldencheatlegendary", 419, 0x0000), // Legendary Trigger Happy
    ("pandoranguardlegendary", 430, 0x0000), // Legendary Chop Chop
    ("landshark_sidekick", 505, 0x0000), // Terrabite
    ("gillgrunt_sidekick", 514, 0x0000), // Gill Runt
    ("goldencheat_sidekick", 519, 0x0000), // Trigger Snappy
    ("stealthelf_sidekick", 526, 0x0000), // Whisper Elf
    ("titanlife_sidekick", 540, 0x0000), // Barkley
    ("whale_sidekick", 541, 0x0000), // Thumpling
    ("giantmagic_sidekick", 542, 0x0000), // Mini Jini
    ("cyclopsgiant_sidekick", 543, 0x0000), // Eye Small
];

/// The figure one of Giants' Collection pictures shows, by the picture's
/// name.
pub fn giants(source: &str) -> Option<(u16, u16)> {
    GIANTS.iter().find(|(name, _, _)| *name == source).map(|&(_, id, variant)| (id, variant))
}

/// Skylanders Spyro's Adventure's versus screen, which shows each Skylander
/// whole, 512 x 512, as `<code name>_vs`: the code name and the figure's id.
/// Each is the plain figure, the only kind the first game reads. Its code
/// names are not all Giants' ("waterdragon" is Zap here), so the table was
/// checked by eye.
pub const SPYROS_ADVENTURE: &[(&str, u16)] = &[
    ("airdragon", 0),                // Whirlwind
    ("griffin", 1),                  // Sonic Boom
    ("skyturtle", 2),                // Warnado
    ("stormgiant", 3),               // Lightning Rod
    ("rockdragon", 4),               // Bash
    ("landshark", 5),                // Terrafin
    ("dinorang", 6),                 // Dino Rang
    ("gemgolem", 7),                 // Prism Break
    ("phoenixdragon", 8),            // Sunburn
    ("eruptor", 9),                  // Eruptor
    ("flameknight", 10),             // Ignitor
    ("fireelf", 11),                 // Flameslinger
    ("waterdragon", 12),             // Zap
    ("crustbuckler", 13),            // Wham Shell
    ("gillgrunt", 14),               // Gill Grunt
    ("yeti", 15),                    // Slam Bam
    ("spyrojr", 16),                 // Spyro
    ("skullorc", 17),                // Voodood
    ("tikiwizard", 18),              // Double Trouble
    ("goldencheat", 19),             // Trigger Happy
    ("metaldragon", 20),             // Drobot
    ("drillbot", 21),                // Drill Sergeant
    ("bombtroll", 22),               // Boomer
    ("forcefieldgrub", 23),          // Wrecking Ball
    ("plantdragon", 24),             // Camo
    ("bambazooker", 25),             // Zook
    ("stealthelf", 26),              // Stealth Elf
    ("stumpsmashent", 27),           // Stump Smash
    ("darkspyro", 28),               // Dark Spyro
    ("shadowmaid", 29),              // Hex
    ("pandoranguard", 30),           // Chop Chop
    ("ghosteater", 31),              // Ghost Roaster
    ("cynder", 32),                  // Cynder
    ("legendaryrockdragon", 404),    // Legendary Bash
    ("legendaryspyro", 416),         // Legendary Spyro
    ("legendarygoldencheat", 419),   // Legendary Trigger Happy
    ("legendarypandoranguard", 430), // Legendary Chop Chop
];

/// The figure one of Spyro's Adventure's versus pictures shows, by the
/// picture's name.
pub fn spyros_adventure(source: &str) -> Option<u16> {
    let code = source.strip_suffix("_vs")?;
    SPYROS_ADVENTURE.iter().find(|(name, _)| *name == code).map(|&(_, id)| id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn giants_pictures_are_told_by_their_names() {
        assert_eq!(giants("airdragon"), Some((0, 0x0000)));
        assert_eq!(giants("gilgrunt_series2"), Some((14, 0x1801)));
        assert_eq!(giants("eaglelightcore"), Some((100, 0x1206)));
        assert_eq!(giants("darkspyro"), Some((28, 0x0000)));
        assert_eq!(giants("magicitem_cannon"), Some((208, 0x1206)));
        assert_eq!(giants("cyclopsgiant_sidekick"), Some((543, 0x0000)));
        assert_eq!(giants("toyportrait_bgowned"), None);
        assert_eq!(giants("Whirlwind_WiiPortrait"), None);
    }

    #[test]
    fn trap_team_pictures_are_told_by_their_archive_names() {
        let named = |path: &str| trap_team(path);
        assert_eq!(
            named("C:/tfb/build/wiiu/ui_sky4/collection_images/skylander_toys/2014/2014_skylanders_regular/462_12288_snapshot.png/0x1.png.igb.tex.igz"),
            Some("462-3000".to_string())
        );
        assert_eq!(named("x/ui_sky4/collection_images/skylander_toys/2011/2011_skylanders/16_0_spyro.png/0x2.igz"), Some("16-0000".to_string()));
        assert_eq!(named("x/ui_sky4/collection_images/traps/square/217_12291.png/0x3.igz"), Some("217-3003".to_string()));
        assert_eq!(named("x/collection_images/villains/captured/1001_villaincaptured.png/0x4.igz"), Some("villain-1001".to_string()));
        assert_eq!(named("x/collection_images/villains/escaped/1046_villainescaped.png/0x5.igz"), Some("villain-1046-loose".to_string()));
        assert_eq!(named("x/ui_sky4/collection_images/traps/square/square_highlight.png/0x6.igz"), None);
        assert_eq!(named("x/ui_sky4/trappersight/png/elementalvignette_life_left.png/0x7.igz"), None);
        assert_eq!(named("c:/tfb/build/wiiu/levels/criminal_audio_sky4/x.wav/0x8.wav.hz.wav.enc"), None);
    }

    #[test]
    fn pictures_are_told_by_their_names() {
        assert_eq!(figure("Spyro_WiiPortrait"), Some((16, 0x0000)));
        assert_eq!(figure("Spyro2012_WiiPortrait"), Some((16, 0x1801)));
        assert_eq!(figure("LegendarySpryo_WiiPortrait"), Some((416, 0x0000)));
        assert_eq!(figure("CatGryphon_WiiPortrait"), Some((3000, 0x0000)));
        assert_eq!(figure("Body_Aviator_Combo"), Some((2000, 0)));
        assert_eq!(figure("Legs_Pirate_Combo"), Some((1015, 0)));
        assert_eq!(figure("Belt_Aviator_Combo"), None);
        assert_eq!(figure("FlockedEruptor_WiiPortrait"), None);
    }

    #[test]
    fn spyros_adventure_pictures_are_told_by_their_names() {
        assert_eq!(spyros_adventure("skyturtle_vs"), Some(2));
        assert_eq!(spyros_adventure("waterdragon_vs"), Some(12));
        assert_eq!(spyros_adventure("legendaryspyro_vs"), Some(416));
        assert_eq!(spyros_adventure("eruptorevil_hud"), None);
        assert_eq!(spyros_adventure("toy_eruptor_hud"), None);
        assert_eq!(spyros_adventure("eruptor"), None);
    }

    #[test]
    fn no_figure_has_two_pictures() {
        for table in [PORTRAITS, GIANTS] {
            let mut seen = std::collections::HashSet::new();
            for &(name, id, variant) in table {
                assert!(seen.insert((id, variant)), "{name} repeats a figure");
            }
        }
        let mut seen = std::collections::HashSet::new();
        for &(name, id) in SPYROS_ADVENTURE {
            assert!(seen.insert(id), "{name} repeats a figure");
        }
    }
}
