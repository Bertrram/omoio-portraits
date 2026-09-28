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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn no_figure_has_two_pictures() {
        let mut seen = std::collections::HashSet::new();
        for &(name, id, variant) in PORTRAITS {
            assert!(seen.insert((id, variant)), "{name} repeats a figure");
        }
    }
}
