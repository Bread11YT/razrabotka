use macroquad::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ══════════════════════════════════════════════════════════════════
// КОНСТАНТЫ
// ══════════════════════════════════════════════════════════════════
const W: f32 = 960.0;
const H: f32 = 540.0;
const HERO_ROW_Y: f32 = 340.0;
const GROUP_START_X: f32 = 70.0;
const GROUP_MAX_X: f32 = 480.0;
const SLOT_OFFSETS: [f32; 5] = [0.0, 55.0, 110.0, 165.0, 220.0];
const SAVE_KEY: &str = "pyatorka_v3";

// ══════════════════════════════════════════════════════════════════
// ENUMS
// ══════════════════════════════════════════════════════════════════
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
enum Tag { Fire, Ice, Holy, Shadow, Storm, Warrior, Mage, Support }

impl Tag {
    fn name(&self) -> &'static str {
        match self {
            Tag::Fire => "Огонь", Tag::Ice => "Лёд", Tag::Holy => "Свет",
            Tag::Shadow => "Тьма", Tag::Storm => "Шторм", Tag::Warrior => "Воин",
            Tag::Mage => "Маг", Tag::Support => "Оплот",
        }
    }
    fn color(&self) -> Color {
        match self {
            Tag::Fire => Color::new(1.0, 0.42, 0.21, 1.0),
            Tag::Ice => Color::new(0.36, 0.78, 1.0, 1.0),
            Tag::Holy => Color::new(1.0, 0.85, 0.24, 1.0),
            Tag::Shadow => Color::new(0.63, 0.42, 1.0, 1.0),
            Tag::Storm => Color::new(0.30, 0.82, 0.88, 1.0),
            Tag::Warrior => Color::new(0.88, 0.36, 0.36, 1.0),
            Tag::Mage => Color::new(0.36, 0.56, 0.88, 1.0),
            Tag::Support => Color::new(0.36, 0.88, 0.54, 1.0),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum HeroKind { Melee, Proj, Chain, Heal }

#[derive(Clone, Copy, PartialEq)]
enum Phase { Prep, Battle, Reward, GameOver }

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
enum TargetMode { Nearest, Weakest, Boss }

#[derive(Clone, Copy, PartialEq)]
enum EnemyKind { Normal, Fast, Tank, Archer, Bomber, Shield, Healer, Boss }

// ══════════════════════════════════════════════════════════════════
// ОПРЕДЕЛЕНИЯ
// ══════════════════════════════════════════════════════════════════
struct HeroDef {
    id: &'static str, name: &'static str,
    tags: &'static [Tag], color: Color,
    base_hp: f32, base_atk: f32, base_aspd: f32, base_range: f32,
    kind: HeroKind, splash: f32, base_crit: f32,
    ult_name: &'static str, ult_desc: &'static str,
}

fn hero_defs() -> Vec<HeroDef> {
    use Tag::*;
    vec![
        HeroDef { id: "pepel", name: "Пепел", tags: &[Fire, Warrior], color: Color::new(1.0, 0.42, 0.21, 1.0),
            base_hp: 280.0, base_atk: 26.0, base_aspd: 1.15, base_range: 130.0, kind: HeroKind::Melee,
            splash: 80.0, base_crit: 0.05, ult_name: "Огненная волна", ult_desc: "AoE 3x ATK" },
        HeroDef { id: "iney", name: "Иней", tags: &[Ice, Mage], color: Color::new(0.36, 0.78, 1.0, 1.0),
            base_hp: 160.0, base_atk: 18.0, base_aspd: 0.95, base_range: 380.0, kind: HeroKind::Proj,
            splash: 0.0, base_crit: 0.05, ult_name: "Ледяной шторм", ult_desc: "Замедление всех" },
        HeroDef { id: "lira", name: "Лира", tags: &[Holy, Support], color: Color::new(1.0, 0.85, 0.24, 1.0),
            base_hp: 180.0, base_atk: 0.0, base_aspd: 0.85, base_range: 0.0, kind: HeroKind::Heal,
            splash: 0.0, base_crit: 0.0, ult_name: "Массовое лечение", ult_desc: "+40% HP команде" },
        HeroDef { id: "klyk", name: "Клык", tags: &[Shadow, Warrior], color: Color::new(0.63, 0.42, 1.0, 1.0),
            base_hp: 200.0, base_atk: 15.0, base_aspd: 2.3, base_range: 110.0, kind: HeroKind::Melee,
            splash: 0.0, base_crit: 0.25, ult_name: "Теневой удар", ult_desc: "4x ATK по цели" },
        HeroDef { id: "grom", name: "Гром", tags: &[Storm, Mage], color: Color::new(0.30, 0.82, 0.88, 1.0),
            base_hp: 170.0, base_atk: 24.0, base_aspd: 0.75, base_range: 400.0, kind: HeroKind::Chain,
            splash: 0.0, base_crit: 0.08, ult_name: "Шторм", ult_desc: "6 случайных врагов" },
        HeroDef { id: "titan", name: "Титан", tags: &[Warrior, Ice], color: Color::new(0.48, 0.71, 0.83, 1.0),
            base_hp: 420.0, base_atk: 18.0, base_aspd: 0.7, base_range: 110.0, kind: HeroKind::Melee,
            splash: 0.0, base_crit: 0.03, ult_name: "Ледяная броня", ult_desc: "Щит 50% HP + AoE" },
        HeroDef { id: "vedma", name: "Ведьма", tags: &[Shadow, Mage, Fire], color: Color::new(0.69, 0.25, 1.0, 1.0),
            base_hp: 140.0, base_atk: 14.0, base_aspd: 1.1, base_range: 360.0, kind: HeroKind::Proj,
            splash: 0.0, base_crit: 0.10, ult_name: "Проклятие", ult_desc: "Поджог+замедление" },
        HeroDef { id: "lun", name: "Лун", tags: &[Holy, Storm], color: Color::new(0.67, 0.85, 1.0, 1.0),
            base_hp: 170.0, base_atk: 22.0, base_aspd: 0.9, base_range: 340.0, kind: HeroKind::Chain,
            splash: 0.0, base_crit: 0.08, ult_name: "Лунный луч", ult_desc: "AoE + heal 30%" },
        HeroDef { id: "valk", name: "Валькирия", tags: &[Holy, Warrior], color: Color::new(1.0, 0.84, 0.0, 1.0),
            base_hp: 240.0, base_atk: 24.0, base_aspd: 1.3, base_range: 120.0, kind: HeroKind::Melee,
            splash: 60.0, base_crit: 0.10, ult_name: "Божественный удар", ult_desc: "AoE + heal 15%" },
        HeroDef { id: "druid", name: "Друид", tags: &[Support, Fire], color: Color::new(0.56, 0.82, 0.44, 1.0),
            base_hp: 200.0, base_atk: 10.0, base_aspd: 1.0, base_range: 300.0, kind: HeroKind::Proj,
            splash: 0.0, base_crit: 0.05, ult_name: "Регенерация", ult_desc: "Лечение + поджог" },
        HeroDef { id: "rogue", name: "Разбойник", tags: &[Shadow, Warrior], color: Color::new(0.4, 0.4, 0.4, 1.0),
            base_hp: 170.0, base_atk: 22.0, base_aspd: 1.8, base_range: 100.0, kind: HeroKind::Melee,
            splash: 0.0, base_crit: 0.40, ult_name: "Смертельный удар", ult_desc: "Крит 500%" },
        HeroDef { id: "monk", name: "Монах", tags: &[Holy, Support, Warrior], color: Color::new(1.0, 0.67, 0.27, 1.0),
            base_hp: 230.0, base_atk: 16.0, base_aspd: 1.2, base_range: 120.0, kind: HeroKind::Melee,
            splash: 50.0, base_crit: 0.08, ult_name: "Просветление", ult_desc: "Лечение + щит" },
    ]
}

struct EnemyDef {
    hp: f32, spd: f32, atk: f32, aspd: f32, range: f32, r: f32,
    color: Color, name: &'static str, kind: EnemyKind, shield: f32,
}

fn enemy_def(kind: EnemyKind) -> EnemyDef {
    match kind {
        EnemyKind::Normal => EnemyDef { hp: 60.0, spd: 70.0, atk: 8.0, aspd: 0.8, range: 50.0, r: 15.0, color: Color::new(0.88, 0.36, 0.36, 1.0), name: "Враг", kind: EnemyKind::Normal, shield: 0.0 },
        EnemyKind::Fast   => EnemyDef { hp: 35.0, spd: 115.0, atk: 5.0, aspd: 1.3, range: 45.0, r: 12.0, color: Color::new(0.49, 0.88, 0.51, 1.0), name: "Бегун", kind: EnemyKind::Fast, shield: 0.0 },
        EnemyKind::Tank   => EnemyDef { hp: 150.0, spd: 45.0, atk: 13.0, aspd: 0.6, range: 55.0, r: 20.0, color: Color::new(0.79, 0.55, 0.29, 1.0), name: "Танк", kind: EnemyKind::Tank, shield: 0.0 },
        EnemyKind::Archer => EnemyDef { hp: 45.0, spd: 55.0, atk: 12.0, aspd: 0.7, range: 280.0, r: 14.0, color: Color::new(0.63, 0.38, 0.82, 1.0), name: "Лучник", kind: EnemyKind::Archer, shield: 0.0 },
        EnemyKind::Bomber => EnemyDef { hp: 40.0, spd: 120.0, atk: 25.0, aspd: 0.0, range: 30.0, r: 13.0, color: Color::new(1.0, 0.67, 0.0, 1.0), name: "Бомбер", kind: EnemyKind::Bomber, shield: 0.0 },
        EnemyKind::Shield => EnemyDef { hp: 120.0, spd: 50.0, atk: 10.0, aspd: 0.7, range: 55.0, r: 19.0, color: Color::new(0.53, 0.6, 0.67, 1.0), name: "Щитоносец", kind: EnemyKind::Shield, shield: 80.0 },
        EnemyKind::Healer => EnemyDef { hp: 70.0, spd: 55.0, atk: 0.0, aspd: 0.9, range: 200.0, r: 14.0, color: Color::new(0.4, 1.0, 0.6, 1.0), name: "Лекарь", kind: EnemyKind::Healer, shield: 0.0 },
        EnemyKind::Boss   => EnemyDef { hp: 800.0, spd: 40.0, atk: 30.0, aspd: 0.8, range: 75.0, r: 34.0, color: Color::new(0.88, 0.29, 0.54, 1.0), name: "БОСС", kind: EnemyKind::Boss, shield: 0.0 },
    }
}

struct GearDef { id: &'static str, name: &'static str, slot: usize, set: usize,
    atk_p: f32, hp_p: f32, aspd_p: f32, crit: f32, regen: f32 }

fn gear_defs() -> Vec<GearDef> {
    // slot: 0=weapon, 1=armor, 2=charm; set: 0=warrior, 1=mage, 2=shadow, 3=holy
    vec![
        GearDef { id:"sword", name:"Клинок", slot:0, set:0, atk_p:0.10, hp_p:0.0, aspd_p:0.0, crit:0.0, regen:0.0 },
        GearDef { id:"plate", name:"Латы", slot:1, set:0, atk_p:0.0, hp_p:0.15, aspd_p:0.0, crit:0.0, regen:0.0 },
        GearDef { id:"warhelm", name:"Шлем", slot:2, set:0, atk_p:0.05, hp_p:0.10, aspd_p:0.0, crit:0.0, regen:0.0 },
        GearDef { id:"staff", name:"Посох", slot:0, set:1, atk_p:0.12, hp_p:0.0, aspd_p:0.0, crit:0.0, regen:0.0 },
        GearDef { id:"robe", name:"Мантия", slot:1, set:1, atk_p:0.0, hp_p:0.10, aspd_p:0.0, crit:0.0, regen:0.0 },
        GearDef { id:"orb", name:"Сфера", slot:2, set:1, atk_p:0.08, hp_p:0.0, aspd_p:0.0, crit:0.05, regen:0.0 },
        GearDef { id:"dagger", name:"Кинжал", slot:0, set:2, atk_p:0.0, hp_p:0.0, aspd_p:0.15, crit:0.10, regen:0.0 },
        GearDef { id:"cloak", name:"Плащ", slot:1, set:2, atk_p:0.0, hp_p:0.08, aspd_p:0.0, crit:0.05, regen:0.0 },
        GearDef { id:"ring", name:"Кольцо", slot:2, set:2, atk_p:0.0, hp_p:0.0, aspd_p:0.0, crit:0.10, regen:0.0 },
        GearDef { id:"mace", name:"Булава", slot:0, set:3, atk_p:0.10, hp_p:0.0, aspd_p:0.0, crit:0.0, regen:0.0 },
        GearDef { id:"aegis", name:"Эгида", slot:1, set:3, atk_p:0.0, hp_p:0.12, aspd_p:0.0, crit:0.0, regen:0.003 },
        GearDef { id:"amulet", name:"Амулет", slot:2, set:3, atk_p:0.0, hp_p:0.0, aspd_p:0.0, crit:0.0, regen:0.005 },
    ]
}

struct SetDef { name: &'static str, color: Color, b2: GearStats, b3: GearStats }
#[derive(Default, Clone, Copy)]
struct GearStats { atk_p: f32, hp_p: f32, aspd_p: f32, crit: f32, regen: f32 }
impl Default for SetDef {
    fn default() -> Self { SetDef { name: "", color: WHITE, b2: GearStats::default(), b3: GearStats::default() } }
}
fn set_defs() -> Vec<SetDef> {
    vec![
        SetDef { name:"Воин", color: Color::new(0.88,0.36,0.36,1.0),
            b2: GearStats{ hp_p: 0.20, ..Default::default() },
            b3: GearStats{ hp_p: 0.40, atk_p: 0.15, ..Default::default() } },
        SetDef { name:"Маг", color: Color::new(0.36,0.56,0.88,1.0),
            b2: GearStats{ atk_p: 0.20, ..Default::default() },
            b3: GearStats{ atk_p: 0.35, crit: 0.15, ..Default::default() } },
        SetDef { name:"Тень", color: Color::new(0.63,0.42,1.0,1.0),
            b2: GearStats{ crit: 0.15, ..Default::default() },
            b3: GearStats{ crit: 0.30, aspd_p: 0.20, ..Default::default() } },
        SetDef { name:"Свет", color: Color::new(1.0,0.85,0.24,1.0),
            b2: GearStats{ regen: 0.005, hp_p: 0.10, ..Default::default() },
            b3: GearStats{ regen: 0.015, hp_p: 0.25, ..Default::default() } },
    ]
}

const SYNERGY: [(Tag, &str, u32, &str); 8] = [
    (Tag::Fire, "Пламя", 2, "Атаки поджигают"),
    (Tag::Ice, "Мороз", 2, "Атаки замедляют"),
    (Tag::Holy, "Благодать", 2, "+2% HP/сек"),
    (Tag::Shadow, "Тень", 2, "+20% крит"),
    (Tag::Storm, "Шторм", 2, "+20% скор. атаки"),
    (Tag::Warrior, "Строй", 2, "+25% макс. HP"),
    (Tag::Mage, "Чародейство", 2, "+25% урона"),
    (Tag::Support, "Забота", 2, "+50% лечение"),
];

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
enum UpgradeKind { Dmg, Hp, Aspd, Regen, Crit, Ability, Heal, Gems, LevelUp }

struct Upgrade { name: &'static str, kind: UpgradeKind, desc: &'static str, icon: &'static str }

fn upgrades() -> Vec<Upgrade> {
    use UpgradeKind::*;
    vec![
        Upgrade { name:"Ярость", kind:Dmg, desc:"+12% урон", icon:"+" },
        Upgrade { name:"Стойкость", kind:Hp, desc:"+15% HP", icon:"+" },
        Upgrade { name:"Скорость", kind:Aspd, desc:"+12% скор.атк", icon:"+" },
        Upgrade { name:"Регенерация", kind:Regen, desc:"+0.5% HP/с", icon:"+" },
        Upgrade { name:"Удача", kind:Crit, desc:"+10% крит", icon:"+" },
        Upgrade { name:"Мастерство", kind:Ability, desc:"+20% ульт", icon:"+" },
        Upgrade { name:"Исцеление", kind:Heal, desc:"Полное лечение", icon:"+" },
        Upgrade { name:"Жадность", kind:Gems, desc:"+200 💎", icon:"+" },
        Upgrade { name:"Закалка", kind:LevelUp, desc:"Герой +2 ур.", icon:"+" },
    ]
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum MetaId { Hp, Atk, Aspd, Crit, StartGems }

struct MetaDef { id: MetaId, name: &'static str, desc: &'static str, max: u32 }
fn meta_defs() -> Vec<MetaDef> {
    vec![
        MetaDef { id: MetaId::Hp, name:"Стойкость", desc:"+10% HP героям", max:10 },
        MetaDef { id: MetaId::Atk, name:"Мощь", desc:"+8% урон", max:10 },
        MetaDef { id: MetaId::Aspd, name:"Быстрота", desc:"+6% скор.атк", max:10 },
        MetaDef { id: MetaId::Crit, name:"Меткость", desc:"+3% крит", max:10 },
        MetaDef { id: MetaId::StartGems, name:"Золото", desc:"+50 старт.💎", max:5 },
    ]
}
fn meta_cost(id: MetaId, lvl: u32) -> u32 {
    match id {
        MetaId::Hp | MetaId::Atk | MetaId::Aspd => 10 + lvl * 5,
        MetaId::Crit => 15 + lvl * 6,
        MetaId::StartGems => 30 + lvl * 20,
    }
}

// ══════════════════════════════════════════════════════════════════
// СТРУКТУРЫ СОСТОЯНИЯ
// ══════════════════════════════════════════════════════════════════
#[derive(Clone, Serialize, Deserialize)]
struct Item { uid: u32, def_id: usize, level: u32 }

#[derive(Clone, Serialize, Deserialize)]
struct HeroData {
    level: u32, shards: u32,
    gear: [Option<u32>; 3],
}
impl Default for HeroData {
    fn default() -> Self { HeroData { level: 1, shards: 0, gear: [None, None, None] } }
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct MetaUpgrades {
    hp: u32, atk: u32, aspd: u32, crit: u32, start_gems: u32,
}
impl MetaUpgrades {
    fn get(&self, id: MetaId) -> u32 {
        match id {
            MetaId::Hp => self.hp, MetaId::Atk => self.atk, MetaId::Aspd => self.aspd,
            MetaId::Crit => self.crit, MetaId::StartGems => self.start_gems,
        }
    }
    fn bump(&mut self, id: MetaId) {
        match id {
            MetaId::Hp => self.hp += 1, MetaId::Atk => self.atk += 1, MetaId::Aspd => self.aspd += 1,
            MetaId::Crit => self.crit += 1, MetaId::StartGems => self.start_gems += 1,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Pity { count: u32, legendary_count: u32 }
impl Default for Pity { fn default() -> Self { Pity { count: 0, legendary_count: 0 } } }

#[derive(Clone, Serialize, Deserialize)]
struct Settings {
    auto_next: bool, sound: bool, target: TargetMode,
}
impl Default for Settings {
    fn default() -> Self { Settings { auto_next: false, sound: true, target: TargetMode::Nearest } }
}

#[derive(Clone, Serialize, Deserialize)]
struct SaveData {
    ver: u32, roster: Vec<usize>, hero_data: HashMap<usize, HeroData>,
    inventory: Vec<Item>, gems: u32, souls: u32, wave_record: u32,
    meta_upgrades: MetaUpgrades, pity: Pity, settings: Settings,
    uid_next: u32, first_rewards: [bool; 3], last_daily: String,
}

#[derive(Clone)]
struct Hero {
    def_idx: usize, slot: usize, level: u32,
    x: f32, y: f32,
    hp: f32, max_hp: f32, atk: f32, aspd: f32, range: f32, crit: f32, regen: f32, shield: f32,
    ult_charge: f32, ult_max: f32, ult_active: f32,
    cd: f32, flash: f32, attack_anim: f32, moving: bool,
    gear_uids: [Option<u32>; 3],
}

#[derive(Clone)]
struct Enemy {
    x: f32, y: f32, hp: f32, max_hp: f32,
    speed: f32, atk: f32, aspd: f32, range: f32, r: f32,
    color: Color, kind: EnemyKind, name: &'static str,
    cd: f32, burn: f32, slow: f32, flash: f32,
    shield_hp: f32, elite: bool, boss_phase: u8,
    telegraph_cd: f32, spawn_cd: f32,
}

#[derive(Clone)]
struct Projectile { x: f32, y: f32, tx: f32, ty: f32, speed: f32, dmg: f32, color: Color, r: f32, target_idx: usize, from_hero: bool }
#[derive(Clone)]
struct Beam { x1: f32, y1: f32, x2: f32, y2: f32, color: Color, life: f32, max: f32 }
#[derive(Clone)]
struct Particle { x: f32, y: f32, vx: f32, vy: f32, life: f32, max: f32, color: Color, r: f32 }
#[derive(Clone)]
struct FloatText { x: f32, y: f32, text: String, color: Color, size: f32, life: f32, max: f32 }
#[derive(Clone)]
struct Telegraph { x: f32, y: f32, r: f32, timer: f32, max_timer: f32, dmg: f32 }
#[derive(Clone)]
struct Shockwave { x: f32, y: f32, max_r: f32, color: Color, life: f32, max: f32 }
#[derive(Clone)]
struct SpawnEvent { delay: f32, kind: EnemyKind, elite: bool }

#[derive(Default, Clone, Copy)]
struct Mods { dmg_mul: f32, hp_mul: f32, aspd_mul: f32, ability_mul: f32, regen: f32, crit_add: f32 }
impl Mods {
    fn reset(&mut self) {
        self.dmg_mul = 1.0; self.hp_mul = 1.0; self.aspd_mul = 1.0;
        self.ability_mul = 1.0; self.regen = 0.0; self.crit_add = 0.0;
    }
}

// UI Modal
#[derive(Clone, PartialEq)]
enum Modal {
    None, Reward(Vec<Upgrade>), Banner, Roster, HeroDetail(usize),
    Merge, Meta, GameOver { souls: u32, wave: u32, avg_lvl: u32 },
    GearPull(Item, String), ShardPull(usize, u32, bool),
}

// ══════════════════════════════════════════════════════════════════
// САМА ИГРА
// ══════════════════════════════════════════════════════════════════
struct Game {
    font: Font,
    hero_defs: Vec<HeroDef>,
    gear_defs: Vec<GearDef>,
    set_defs: Vec<SetDef>,
    upgrades: Vec<Upgrade>,
    meta_defs: Vec<MetaDef>,

    // persistent
    roster: Vec<usize>,
    hero_data: HashMap<usize, HeroData>,
    inventory: Vec<Item>,
    gems: u32, souls: u32, wave_record: u32,
    meta_up: MetaUpgrades,
    pity: Pity, settings: Settings,
    uid_next: u32,
    first_rewards: [bool; 3],
    last_daily: String,

    // per-run
    phase: Phase, wave: u32, time_scale: f32, paused: bool,
    group_x: f32, group_moving: bool,
    heroes: Vec<Hero>,
    enemies: Vec<Enemy>,
    projectiles: Vec<Projectile>,
    beams: Vec<Beam>, particles: Vec<Particle>, floats: Vec<FloatText>,
    telegraphs: Vec<Telegraph>, shockwaves: Vec<Shockwave>,
    spawn_queue: Vec<SpawnEvent>, spawn_timer: f32, time: f32,
    mods: Mods,
    synergies: HashMap<Tag, bool>,
    shake_t: f32, shake_a: f32,

    modal: Modal,
    click: Option<Vec2>,
}

impl Game {
    fn new(font: Font) -> Self {
        let mut g = Game {
            font,
            hero_defs: hero_defs(),
            gear_defs: gear_defs(),
            set_defs: set_defs(),
            upgrades: upgrades(),
            meta_defs: meta_defs(),
            roster: vec![0, 1, 2, 3, 4],
            hero_data: HashMap::new(),
            inventory: Vec::new(),
            gems: 300, souls: 0, wave_record: 0,
            meta_up: MetaUpgrades::default(),
            pity: Pity::default(),
            settings: Settings::default(),
            uid_next: 1,
            first_rewards: [false; 3],
            last_daily: String::new(),
            phase: Phase::Prep, wave: 0, time_scale: 1.0, paused: false,
            group_x: GROUP_START_X, group_moving: false,
            heroes: Vec::new(),
            enemies: Vec::new(), projectiles: Vec::new(),
            beams: Vec::new(), particles: Vec::new(), floats: Vec::new(),
            telegraphs: Vec::new(), shockwaves: Vec::new(),
            spawn_queue: Vec::new(), spawn_timer: 0.0, time: 0.0,
            mods: { let mut m = Mods::default(); m.reset(); m },
            synergies: HashMap::new(),
            shake_t: 0.0, shake_a: 0.0,
            modal: Modal::None,
            click: None,
        };
        for i in 0..g.hero_defs.len() { g.hero_data.insert(i, HeroData::default()); }
        g.rebuild_heroes();
        g
    }

    fn rebuild_heroes(&mut self) {
        self.heroes.clear();
        for (slot, &def_idx) in self.roster.iter().enumerate() {
            let hd = self.hero_data.get(&def_idx).cloned().unwrap_or_default();
            let def = &self.hero_defs[def_idx];
            let gear_uids = hd.gear;
            self.heroes.push(Hero {
                def_idx, slot, level: hd.level,
                x: GROUP_START_X + SLOT_OFFSETS[slot], y: HERO_ROW_Y,
                hp: 0.0, max_hp: 0.0, atk: 0.0, aspd: 1.0, range: 0.0,
                crit: 0.0, regen: 0.0, shield: 0.0,
                ult_charge: 0.0, ult_max: 100.0, ult_active: 0.0,
                cd: 0.0, flash: 0.0, attack_anim: 0.0, moving: false,
                gear_uids,
            });
            let _ = def;
        }
        self.recalc_heroes();
        for h in &mut self.heroes { if h.hp <= 0.0 { h.hp = h.max_hp; } }
    }

    fn gear_stats(&self, hero: &Hero) -> (GearStats, [u32; 4]) {
        let mut s = GearStats::default();
        let mut counts = [0u32; 4];
        for uid in hero.gear_uids.iter().flatten() {
            if let Some(item) = self.inventory.iter().find(|i| i.uid == *uid) {
                if let Some(def) = self.gear_defs.get(item.def_id) {
                    let mult = 1.5_f32.powi(item.level as i32 - 1);
                    s.atk_p += def.atk_p * mult;
                    s.hp_p += def.hp_p * mult;
                    s.aspd_p += def.aspd_p * mult;
                    s.crit += def.crit * mult;
                    s.regen += def.regen * mult;
                    counts[def.set] += 1;
                }
            }
        }
        for (i, &c) in counts.iter().enumerate() {
            if c >= 2 { let b = self.set_defs[i].b2; s.atk_p += b.atk_p; s.hp_p += b.hp_p; s.aspd_p += b.aspd_p; s.crit += b.crit; s.regen += b.regen; }
            if c >= 3 { let b = self.set_defs[i].b3; s.atk_p += b.atk_p; s.hp_p += b.hp_p; s.aspd_p += b.aspd_p; s.crit += b.crit; s.regen += b.regen; }
        }
        (s, counts)
    }

    fn recalc_heroes(&mut self) {
        // синэргии
        let mut tag_counts: HashMap<Tag, u32> = HashMap::new();
        for h in &self.heroes {
            for &t in self.hero_defs[h.def_idx].tags { *tag_counts.entry(t).or_insert(0) += 1; }
        }
        self.synergies.clear();
        for (tag, _, need, _) in SYNERGY.iter() {
            self.synergies.insert(*tag, *tag_counts.get(tag).unwrap_or(&0) >= *need);
        }

        let m = self.mods;
        let mu = &self.meta_up;
        let meta_hp = 1.0 + mu.hp as f32 * 0.10;
        let meta_atk = 1.0 + mu.atk as f32 * 0.08;
        let meta_aspd = 1.0 + mu.aspd as f32 * 0.06;
        let meta_crit = mu.crit as f32 * 0.03;

        let mut updates: Vec<(usize, f32, f32, f32, f32, f32, f32, f32)> = Vec::new();
        for (i, h) in self.heroes.iter().enumerate() {
            let def = &self.hero_defs[h.def_idx];
            let (gs, _) = self.gear_stats(h);
            let lvl_hp = 1.0 + (h.level as f32 - 1.0) * 0.09;
            let lvl_dmg = 1.0 + (h.level as f32 - 1.0) * 0.08;
            let lvl_aspd = 1.0 + (h.level as f32 - 1.0) * 0.03;

            let mut hp_mul = m.hp_mul * lvl_hp * (1.0 + gs.hp_p) * meta_hp;
            let mut dmg_mul = m.dmg_mul * lvl_dmg * (1.0 + gs.atk_p) * meta_atk;
            let mut aspd_mul = m.aspd_mul * lvl_aspd * (1.0 + gs.aspd_p) * meta_aspd;

            if *self.synergies.get(&Tag::Warrior).unwrap_or(&false) { hp_mul *= 1.25; }
            if *self.synergies.get(&Tag::Storm).unwrap_or(&false) { aspd_mul *= 1.20; }
            if *self.synergies.get(&Tag::Mage).unwrap_or(&false) { dmg_mul *= 1.25; }

            let new_max = (def.base_hp * hp_mul).round();
            let new_atk = def.base_atk * dmg_mul;
            let new_aspd = def.base_aspd * aspd_mul;
            let new_crit = def.base_crit + m.crit_add + gs.crit + meta_crit
                + if *self.synergies.get(&Tag::Shadow).unwrap_or(&false) { 0.20 } else { 0.0 };
            let new_regen = gs.regen + m.regen;

            updates.push((i, new_max, new_atk, new_aspd, new_crit, new_regen, hp_mul, dmg_mul));
        }
        for (i, new_max, new_atk, new_aspd, new_crit, new_regen, _, _) in updates {
            let h = &mut self.heroes[i];
            let old_max = h.max_hp;
            h.max_hp = new_max;
            h.atk = new_atk;
            h.aspd = new_aspd;
            h.crit = new_crit;
            h.regen = new_regen;
            h.range = self.hero_defs[h.def_idx].base_range;
            if old_max <= 0.0 { h.hp = new_max; }
            else { h.hp = (h.hp / old_max) * new_max; }
        }
    }

    // ─────────── Сохранение ───────────
    fn save(&self) {
        let data = SaveData {
            ver: 3,
            roster: self.roster.clone(),
            hero_data: self.hero_data.clone(),
            inventory: self.inventory.clone(),
            gems: self.gems, souls: self.souls, wave_record: self.wave_record,
            meta_upgrades: self.meta_up.clone(),
            pity: self.pity.clone(), settings: self.settings.clone(),
            uid_next: self.uid_next,
            first_rewards: self.first_rewards,
            last_daily: self.last_daily.clone(),
        };
        if let Ok(json) = serde_json::to_string(&data) { save_to_storage(SAVE_KEY, &json); }
    }

    fn load(&mut self) {
        if let Some(json) = load_from_storage(SAVE_KEY) {
            if let Ok(data) = serde_json::from_str::<SaveData>(&json) {
                self.roster = data.roster;
                self.hero_data = data.hero_data;
                for i in 0..self.hero_defs.len() {
                    self.hero_data.entry(i).or_insert_with(HeroData::default);
                }
                self.inventory = data.inventory;
                self.gems = data.gems;
                self.souls = data.souls;
                self.wave_record = data.wave_record;
                self.meta_up = data.meta_upgrades;
                self.pity = data.pity;
                self.settings = data.settings;
                self.uid_next = data.uid_next;
                self.first_rewards = data.first_rewards;
                self.last_daily = data.last_daily;
            }
        } else {
            // стартовый подарок
            self.inventory.push(Item { uid: self.new_uid(), def_id: 0, level: 1 });
            self.inventory.push(Item { uid: self.new_uid(), def_id: 4, level: 1 });
        }
    }

    fn new_uid(&mut self) -> u32 { let u = self.uid_next; self.uid_next += 1; u }

    // ─────────── Хелперы ───────────
    fn add_float(&mut self, x: f32, y: f32, text: impl Into<String>, color: Color, size: f32) {
        self.floats.push(FloatText { x, y, text: text.into(), color, size, life: 0.9, max: 0.9 });
    }
    fn add_particles(&mut self, x: f32, y: f32, color: Color, count: u32) {
        for _ in 0..count {
            let a = rand::gen_range(0.0, std::f32::consts::TAU);
            let s = rand::gen_range(30.0, 160.0);
            self.particles.push(Particle {
                x, y, vx: a.cos() * s, vy: a.sin() * s,
                life: rand::gen_range(0.25, 0.6), max: 0.6,
                color, r: rand::gen_range(1.5, 3.5),
            });
        }
    }
    fn add_beam(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Color, life: f32) {
        self.beams.push(Beam { x1, y1, x2, y2, color, life, max: life });
    }
    fn add_shake(&mut self, t: f32, a: f32) {
        self.shake_t = self.shake_t.max(t);
        self.shake_a = self.shake_a.max(a);
    }
    fn add_shockwave(&mut self, x: f32, y: f32, r: f32, color: Color, life: f32) {
        self.shockwaves.push(Shockwave { x, y, max_r: r, color, life, max: life });
    }

    // ─────────── Урон ───────────
    fn damage_enemy(&mut self, idx: usize, dmg: f32, crit: f32) -> f32 {
        if idx >= self.enemies.len() || self.enemies[idx].hp <= 0.0 { return 0.0; }
        let is_crit = rand::gen_range(0.0_f32, 1.0) < (0.05 + crit);
        let mut final_dmg = dmg * if is_crit { 2.0 } else { 1.0 };

        // щит
        if self.enemies[idx].shield_hp > 0.0 {
            let abs = self.enemies[idx].shield_hp.min(final_dmg);
            self.enemies[idx].shield_hp -= abs;
            final_dmg -= abs;
        }
        self.enemies[idx].hp -= final_dmg;
        self.enemies[idx].flash = 0.12;

        let (ex, ey, er, ecolor, eburn) = (self.enemies[idx].x, self.enemies[idx].y, self.enemies[idx].r, self.enemies[idx].color, self.enemies[idx].burn);
        self.add_float(ex, ey - er - 6.0, format!("{}", final_dmg.round()), if is_crit { Color::new(1.0, 0.85, 0.24, 1.0) } else { WHITE }, if is_crit { 17.0 } else { 13.0 });
        self.add_particles(ex, ey, if is_crit { Color::new(1.0, 0.85, 0.24, 1.0) } else { WHITE }, if is_crit { 8 } else { 3 });

        if *self.synergies.get(&Tag::Fire).unwrap_or(&false) { self.enemies[idx].burn = 3.0; let _ = eburn; }
        if *self.synergies.get(&Tag::Ice).unwrap_or(&false) {
            self.enemies[idx].slow = self.enemies[idx].slow.max(1.5);
        }

        if self.enemies[idx].hp <= 0.0 {
            self.add_particles(ex, ey, ecolor, 12);
            self.add_float(ex, ey, "X", Color::new(1.0, 0.42, 0.42, 1.0), 15.0);
        }
        final_dmg
    }

    fn damage_hero(&mut self, idx: usize, dmg: f32) {
        if idx >= self.heroes.len() || self.heroes[idx].hp <= 0.0 { return; }
        let mut final_dmg = dmg;
        if self.heroes[idx].shield > 0.0 {
            let abs = self.heroes[idx].shield.min(final_dmg);
            self.heroes[idx].shield -= abs;
            final_dmg -= abs;
        }
        self.heroes[idx].hp -= final_dmg;
        self.heroes[idx].flash = 0.12;
        let (hx, hy) = (self.heroes[idx].x, self.heroes[idx].y);
        self.add_float(hx, hy - 24.0, format!("{}", final_dmg.round()), Color::new(1.0, 0.42, 0.42, 1.0), 13.0);
        self.add_particles(hx, hy, Color::new(1.0, 0.42, 0.42, 1.0), 4);
    }

    fn heal_hero(&mut self, idx: usize, amount: f32) {
        if idx >= self.heroes.len() || self.heroes[idx].hp <= 0.0 { return; }
        let before = self.heroes[idx].hp;
        self.heroes[idx].hp = (self.heroes[idx].hp + amount).min(self.heroes[idx].max_hp);
        let healed = (self.heroes[idx].hp - before).round();
        if healed > 0.0 {
            let (hx, hy) = (self.heroes[idx].x, self.heroes[idx].y);
            self.add_float(hx, hy - 28.0, format!("+{}", healed), Color::new(0.36, 0.88, 0.54, 1.0), 13.0);
            self.add_particles(hx, hy, Color::new(0.36, 0.88, 0.54, 1.0), 3);
        }
    }

    // ─────────── Волны ───────────
    fn spawn_wave(&mut self) {
        self.wave += 1;
        self.phase = Phase::Battle;
        self.spawn_queue.clear();
        self.spawn_timer = 0.0;
        self.group_x = GROUP_START_X;
        self.enemies.clear();
        self.projectiles.clear();
        self.telegraphs.clear();
        self.shockwaves.clear();
        for h in &mut self.heroes {
            if h.hp > 0.0 { h.hp = h.max_hp; }
            h.ult_charge = 0.0;
            h.shield = 0.0;
            h.x = self.group_x + SLOT_OFFSETS[h.slot];
            h.y = HERO_ROW_Y;
        }
        let n = self.wave;
        let count = 8 + n * 2;
        let is_elite = n % 10 == 0;
        for i in 0..count {
            let r = rand::gen_range(0.0_f32, 1.0);
            let kind = if r < 0.10 { EnemyKind::Archer }
                else if r < 0.17 { EnemyKind::Bomber }
                else if r < 0.24 { EnemyKind::Shield }
                else if r < 0.30 { EnemyKind::Healer }
                else if r < 0.45 { EnemyKind::Fast }
                else if r < 0.60 { EnemyKind::Tank }
                else { EnemyKind::Normal };
            self.spawn_queue.push(SpawnEvent { delay: i as f32 * 0.36 + rand::gen_range(0.0, 0.3), kind, elite: is_elite });
        }
        if n % 5 == 0 {
            self.spawn_queue.push(SpawnEvent { delay: count as f32 * 0.36 + 1.5, kind: EnemyKind::Boss, elite: is_elite });
        }
        self.spawn_queue.sort_by(|a, b| a.delay.partial_cmp(&b.delay).unwrap());
        self.save();
    }

    fn make_enemy(&self, kind: EnemyKind, wave: u32, elite: bool) -> Enemy {
        let def = enemy_def(kind);
        let scale = 1.0 + (wave as f32 - 1.0) * 0.18 + self.wave_record as f32 * 0.01;
        let elite_mul = if elite { 1.5 } else { 1.0 };
        Enemy {
            x: W + rand::gen_range(20.0, 140.0),
            y: HERO_ROW_Y + rand::gen_range(-180.0, 140.0),
            hp: def.hp * scale * elite_mul, max_hp: def.hp * scale * elite_mul,
            speed: def.spd, atk: def.atk * scale, aspd: def.aspd,
            range: def.range, r: def.r, color: def.color, kind, name: def.name,
            cd: rand::gen_range(0.0, 0.6), burn: 0.0, slow: 0.0, flash: 0.0,
            shield_hp: if def.shield > 0.0 { def.shield * scale } else { 0.0 },
            elite, boss_phase: 1, telegraph_cd: 3.0, spawn_cd: 8.0,
        }
    }

    fn end_wave(&mut self) {
        self.phase = Phase::Reward;
        self.gems += 100 + self.wave * 30;
        if self.wave % 10 == 0 {
            let item = self.roll_gear(2);
            self.inventory.push(item);
        }
        if self.wave > self.wave_record { self.wave_record = self.wave; }
        self.save();

        if self.settings.auto_next {
            let u = self.upgrades[rand::gen_range(0, self.upgrades.len())];
            self.apply_upgrade(u);
            self.phase = Phase::Prep;
            self.spawn_wave();
        } else {
            let mut pool: Vec<Upgrade> = self.upgrades.clone();
            for i in (1..pool.len()).rev() {
                let j = rand::gen_range(0, i + 1);
                pool.swap(i, j);
            }
            pool.truncate(3);
            self.modal = Modal::Reward(pool);
        }
    }

    fn apply_upgrade(&mut self, u: Upgrade) {
        match u.kind {
            UpgradeKind::Dmg => self.mods.dmg_mul *= 1.12,
            UpgradeKind::Hp => self.mods.hp_mul *= 1.15,
            UpgradeKind::Aspd => self.mods.aspd_mul *= 1.12,
            UpgradeKind::Regen => self.mods.regen += 0.005,
            UpgradeKind::Crit => self.mods.crit_add += 0.10,
            UpgradeKind::Ability => self.mods.ability_mul *= 1.20,
            UpgradeKind::Heal => { for h in &mut self.heroes { if h.hp > 0.0 { h.hp = h.max_hp; } } }
            UpgradeKind::Gems => self.gems += 200,
            UpgradeKind::LevelUp => {
                if let Some(&idx) = self.roster.get(rand::gen_range(0, self.roster.len())) {
                    if let Some(hd) = self.hero_data.get_mut(&idx) { hd.level += 2; }
                }
            }
        }
        self.recalc_heroes();
    }

    // ─────────── Гача ───────────
    fn roll_gear(&mut self, min_lvl: u32) -> Item {
        let pc = self.pity.count;
        let lc = self.pity.legendary_count;
        let forced = if lc >= 60 { 4 } else if pc >= 20 { 3 } else { 0 };
        let roll = rand::gen_range(0.0_f32, 1.0);
        let tier = if forced > 0 { forced }
            else if roll < 0.55 { 1 } else if roll < 0.85 { 2 } else if roll < 0.97 { 3 } else { 4 };
        let level = min_lvl + (tier - 1);
        let def_id = rand::gen_range(0, self.gear_defs.len());
        let uid = self.new_uid();
        Item { uid, def_id, level }
    }

    fn pull_gear(&mut self) {
        if self.gems < 100 { return; }
        self.gems -= 100;
        self.pity.count += 1;
        let item = self.roll_gear(1);
        let tier = item.level;
        if tier >= 4 { self.pity.legendary_count = 0; } else { self.pity.legendary_count += 1; }
        if self.pity.count >= 20 { self.pity.count = 0; }
        let def = &self.gear_defs[item.def_id];
        let rar = if tier <= 1 { "common" } else if tier == 2 { "rare" } else if tier == 3 { "epic" } else { "legend" };
        self.inventory.push(item.clone());
        self.modal = Modal::GearPull(item, rar.to_string());
        self.save();
    }

    fn pull_shards(&mut self, hero_idx: usize) {
        if self.gems < 150 { return; }
        self.gems -= 150;
        let amt = rand::gen_range(5, 11);
        let mut leveled = false;
        if let Some(hd) = self.hero_data.get_mut(&hero_idx) {
            hd.shards += amt;
            while hd.shards >= 10 { hd.shards -= 10; hd.level += 1; leveled = true; }
        }
        if leveled { self.rebuild_heroes(); }
        self.modal = Modal::ShardPull(hero_idx, amt, leveled);
        self.save();
    }

    // ─────────── Бой ───────────
    fn update(&mut self, dt: f32) {
        if self.paused || !matches!(self.modal, Modal::None) { return; }
        match self.phase {
            Phase::Battle => self.update_battle(dt),
            Phase::Prep => self.update_group_movement(dt),
            _ => {}
        }
        self.update_effects(dt);
        // авто-старт
        if matches!(self.phase, Phase::Prep) && self.settings.auto_next && self.wave > 0 {
            // не реализовано: авто-старт при авто-выборе
        }
    }

    fn update_group_movement(&mut self, dt: f32) {
        for h in &mut self.heroes {
            if h.hp > 0.0 {
                h.x = self.group_x + SLOT_OFFSETS[h.slot];
                h.y = HERO_ROW_Y;
            }
        }
        if self.phase != Phase::Battle { self.group_moving = false; return; }
        if self.enemies.is_empty() {
            if self.group_x < GROUP_MAX_X {
                self.group_x = (self.group_x + 60.0 * dt).min(GROUP_MAX_X);
                self.group_moving = true;
            } else { self.group_moving = false; }
            return;
        }
        let mut nearest_x = f32::MAX;
        for e in &self.enemies { if e.x < nearest_x { nearest_x = e.x; } }
        let mut front: Option<&Hero> = None;
        for h in &self.heroes {
            if h.hp > 0.0 && self.hero_defs[h.def_idx].kind != HeroKind::Heal {
                if front.is_none() || h.slot > front.unwrap().slot { front = Some(h); }
            }
        }
        let should_stop = if let Some(f) = front {
            let reach = self.group_x + SLOT_OFFSETS[f.slot] + f.range;
            nearest_x - 30.0 <= reach
        } else { true };

        if !should_stop && self.group_x < GROUP_MAX_X {
            self.group_x = (self.group_x + 60.0 * dt).min(GROUP_MAX_X);
            self.group_moving = true;
        } else { self.group_moving = false; }
    }

    fn nearest_hero(&self, x: f32, y: f32) -> Option<usize> {
        let mut best: Option<usize> = None;
        let mut best_d = f32::MAX;
        for (i, h) in self.heroes.iter().enumerate() {
            if h.hp <= 0.0 { continue; }
            let d = ((h.x - x).powi(2) + (h.y - y).powi(2)).sqrt();
            if d < best_d { best_d = d; best = Some(i); }
        }
        best
    }

    fn pick_target(&self, hero_idx: usize) -> Option<usize> {
        let h = &self.heroes[hero_idx];
        let mut alive: Vec<usize> = Vec::new();
        for (i, e) in self.enemies.iter().enumerate() {
            if e.hp <= 0.0 { continue; }
            let d = ((e.x - h.x).powi(2) + (e.y - h.y).powi(2)).sqrt();
            if d <= h.range { alive.push(i); }
        }
        if alive.is_empty() { return None; }
        match self.settings.target {
            TargetMode::Weakest => alive.sort_by(|&a, &b| self.enemies[a].hp.partial_cmp(&self.enemies[b].hp).unwrap()),
            TargetMode::Boss => {
                if let Some(&b) = alive.iter().find(|&&i| self.enemies[i].kind == EnemyKind::Boss) { return Some(b); }
                alive.sort_by(|&a, &b| {
                    let da = (self.enemies[a].x - h.x).powi(2) + (self.enemies[a].y - h.y).powi(2);
                    let db = (self.enemies[b].x - h.x).powi(2) + (self.enemies[b].y - h.y).powi(2);
                    da.partial_cmp(&db).unwrap()
                });
            }
            TargetMode::Nearest => alive.sort_by(|&a, &b| {
                let da = (self.enemies[a].x - h.x).powi(2) + (self.enemies[a].y - h.y).powi(2);
                let db = (self.enemies[b].x - h.x).powi(2) + (self.enemies[b].y - h.y).powi(2);
                da.partial_cmp(&db).unwrap()
            }),
        }
        alive.first().copied()
    }

    fn hero_attack(&mut self, hero_idx: usize, target_idx: usize) {
        let h = &self.heroes[hero_idx];
        let def = &self.hero_defs[h.def_idx];
        let (hx, hy) = (h.x, h.y);
        let (tx, ty) = (self.enemies[target_idx].x, self.enemies[target_idx].y);
        let dmg = h.atk;
        let crit = h.crit;
        let kind = def.kind;
        let splash = def.splash;
        let color = def.color;

        self.heroes[hero_idx].attack_anim = 0.2;
        self.heroes[hero_idx].flash = 0.1;

        match kind {
            HeroKind::Melee => {
                if splash > 0.0 {
                    let nearby: Vec<usize> = self.enemies.iter().enumerate()
                        .filter(|(_, e)| e.hp > 0.0 && ((e.x - tx).powi(2) + (e.y - ty).powi(2)).sqrt() <= splash)
                        .map(|(i, _)| i).collect();
                    let mut total = 0.0;
                    for &ei in &nearby { total += self.damage_enemy(ei, dmg * 0.85, crit); }
                    total += self.damage_enemy(target_idx, dmg, crit);
                    self.heroes[hero_idx].ult_charge += total * 0.35;
                    self.add_beam(hx, hy, tx, ty, color, 0.13);
                } else {
                    let dealt = self.damage_enemy(target_idx, dmg, crit);
                    self.heroes[hero_idx].ult_charge += dealt * 0.35;
                    self.add_beam(hx, hy, tx, ty, color, 0.09);
                }
            }
            HeroKind::Proj => {
                self.projectiles.push(Projectile {
                    x: hx, y: hy, tx, ty, speed: 560.0, dmg, color, r: 5.0,
                    target_idx, from_hero: true,
                });
            }
            HeroKind::Chain => {
                let others: Vec<usize> = self.enemies.iter().enumerate()
                    .filter(|(i, e)| *i != target_idx && e.hp > 0.0)
                    .map(|(i, _)| i)
                    .collect();
                let mut sorted = others;
                sorted.sort_by(|&a, &b| {
                    let da = (self.enemies[a].x - tx).powi(2) + (self.enemies[a].y - ty).powi(2);
                    let db = (self.enemies[b].x - tx).powi(2) + (self.enemies[b].y - ty).powi(2);
                    da.partial_cmp(&db).unwrap()
                });
                let targets: Vec<usize> = std::iter::once(target_idx).chain(sorted.into_iter().take(2)).collect();
                let mut total = 0.0;
                for (i, &ti) in targets.iter().enumerate() {
                    total += self.damage_enemy(ti, dmg * (1.0 - i as f32 * 0.28), crit);
                    let (ex, ey) = (self.enemies[ti].x, self.enemies[ti].y);
                    self.add_beam(hx, hy, ex, ey, color, 0.16);
                }
                self.heroes[hero_idx].ult_charge += total * 0.35;
            }
            HeroKind::Heal => {}
        }
    }

    fn cast_ult(&mut self, hero_idx: usize) {
        let h = &self.heroes[hero_idx];
        let def = &self.hero_defs[h.def_idx];
        let (hx, hy) = (h.x, h.y);
        let id = def.id;
        let color = def.color;
        let atk = h.atk;
        let max_hp = h.max_hp;
        let p = self.mods.ability_mul;
        let _ = max_hp;

        self.heroes[hero_idx].ult_charge = 0.0;
        self.heroes[hero_idx].ult_active = 0.4;
        self.add_shake(0.25, 5.0);
        self.add_shockwave(hx, hy, 200.0, color, 0.5);

        match id {
            "pepel" => {
                let targets: Vec<usize> = self.enemies.iter().enumerate()
                    .filter(|(_, e)| e.hp > 0.0 && ((e.x - hx).powi(2) + (e.y - hy).powi(2)).sqrt() < 220.0)
                    .map(|(i, _)| i).collect();
                for ei in targets { self.damage_enemy(ei, atk * 3.0 * p, 0.0); self.enemies[ei].burn = 4.0; }
                self.add_shockwave(hx, hy, 220.0, Color::new(1.0, 0.42, 0.21, 1.0), 0.6);
            }
            "iney" | "vedma" => {
                let all: Vec<usize> = (0..self.enemies.len()).filter(|&i| self.enemies[i].hp > 0.0).collect();
                for ei in all {
                    self.damage_enemy(ei, atk * 1.2 * p, 0.0);
                    self.enemies[ei].slow = 4.0;
                    if id == "vedma" { self.enemies[ei].burn = 4.0; }
                }
                self.add_shockwave(hx, hy, 500.0, color, 0.5);
            }
            "lira" => {
                let count = self.heroes.len();
                for i in 0..count { self.heal_hero(i, self.heroes[i].max_hp * 0.4 * p); }
                self.add_shockwave(hx, hy, 300.0, Color::new(1.0, 0.85, 0.24, 1.0), 0.6);
            }
            "klyk" | "rogue" => {
                if let Some(t) = self.pick_target(hero_idx) {
                    let mult = if id == "rogue" { 5.0 } else { 4.0 };
                    let crit = if id == "rogue" { 10.0 } else { 0.0 };
                    self.damage_enemy(t, atk * mult * p, crit);
                    let (tx, ty) = (self.enemies[t].x, self.enemies[t].y);
                    self.add_beam(hx, hy, tx, ty, color, 0.3);
                }
            }
            "grom" => {
                let mut all: Vec<usize> = (0..self.enemies.len()).filter(|&i| self.enemies[i].hp > 0.0).collect();
                for i in (1..all.len()).rev() { let j = rand::gen_range(0, i + 1); all.swap(i, j); }
                for &ei in all.iter().take(6) {
                    self.damage_enemy(ei, atk * 1.6 * p, 0.0);
                    let (tx, ty) = (self.enemies[ei].x, self.enemies[ei].y);
                    self.add_beam(hx, hy, tx, ty, color, 0.2);
                }
            }
            "titan" => {
                self.heroes[hero_idx].shield = self.heroes[hero_idx].max_hp * 0.5;
                let near: Vec<usize> = self.enemies.iter().enumerate()
                    .filter(|(_, e)| e.hp > 0.0 && ((e.x - hx).powi(2) + (e.y - hy).powi(2)).sqrt() < 220.0)
                    .map(|(i, _)| i).collect();
                for ei in near { self.damage_enemy(ei, atk * 2.0 * p, 0.0); }
                self.add_shockwave(hx, hy, 220.0, color, 0.6);
            }
            "lun" => {
                let near: Vec<usize> = self.enemies.iter().enumerate()
                    .filter(|(_, e)| e.hp > 0.0 && ((e.x - hx).powi(2) + (e.y - hy).powi(2)).sqrt() < 300.0)
                    .map(|(i, _)| i).collect();
                for ei in near { self.damage_enemy(ei, atk * 2.2 * p, 0.0); }
                let mhp = self.heroes[hero_idx].max_hp;
                self.heal_hero(hero_idx, mhp * 0.3);
                self.add_shockwave(hx, hy, 300.0, color, 0.6);
            }
            "valk" => {
                let near: Vec<usize> = self.enemies.iter().enumerate()
                    .filter(|(_, e)| e.hp > 0.0 && ((e.x - hx).powi(2) + (e.y - hy).powi(2)).sqrt() < 200.0)
                    .map(|(i, _)| i).collect();
                for ei in near { self.damage_enemy(ei, atk * 2.5 * p, 0.0); }
                let count = self.heroes.len();
                for i in 0..count { self.heal_hero(i, self.heroes[i].max_hp * 0.15); }
                self.add_shockwave(hx, hy, 200.0, color, 0.6);
            }
            "druid" => {
                let count = self.heroes.len();
                for i in 0..count { self.heal_hero(i, self.heroes[i].max_hp * 0.25 * p); }
                for e in &mut self.enemies { e.burn = 4.0; }
                self.add_shockwave(hx, hy, 400.0, color, 0.6);
            }
            "monk" => {
                let count = self.heroes.len();
                for i in 0..count {
                    let mhp = self.heroes[i].max_hp;
                    self.heal_hero(i, mhp * 0.3 * p);
                    self.heroes[i].shield = self.heroes[i].shield.max(mhp * 0.25);
                }
                self.add_shockwave(hx, hy, 300.0, color, 0.6);
            }
            _ => {}
        }
    }

    fn update_battle(&mut self, dt: f32) {
        self.time += dt;

        // спавн
        self.spawn_timer += dt;
        while !self.spawn_queue.is_empty() && self.spawn_queue[0].delay <= self.spawn_timer {
            let ev = self.spawn_queue.remove(0);
            let e = self.make_enemy(ev.kind, self.wave, ev.elite);
            self.enemies.push(e);
        }

        // holy regen
        if *self.synergies.get(&Tag::Holy).unwrap_or(&false) {
            for h in &mut self.heroes {
                if h.hp > 0.0 { h.hp = (h.hp + h.max_hp * 0.02 * dt).min(h.max_hp); }
            }
        }

        self.update_group_movement(dt);

        // герои
        for hi in 0..self.heroes.len() {
            if self.heroes[hi].hp <= 0.0 { continue; }
            let kind = self.hero_defs[self.heroes[hi].def_idx].kind;
            self.heroes[hi].cd -= dt;
            self.heroes[hi].flash = (self.heroes[hi].flash - dt).max(0.0);
            self.heroes[hi].attack_anim = (self.heroes[hi].attack_anim - dt).max(0.0);
            self.heroes[hi].ult_active = (self.heroes[hi].ult_active - dt).max(0.0);
            self.heroes[hi].moving = self.group_moving;

            // реген
            if self.heroes[hi].regen > 0.0 {
                let regen = self.heroes[hi].regen;
                let mhp = self.heroes[hi].max_hp;
                self.heroes[hi].hp = (self.heroes[hi].hp + mhp * regen * dt).min(mhp);
            }

            // ульта
            if self.heroes[hi].ult_charge >= self.heroes[hi].ult_max && self.heroes[hi].ult_active <= 0.0 {
                self.cast_ult(hi);
            }

            if kind == HeroKind::Heal {
                let mut worst: Option<usize> = None;
                let mut wr = 1.0;
                for (i, a) in self.heroes.iter().enumerate() {
                    if a.hp <= 0.0 { continue; }
                    let r = a.hp / a.max_hp;
                    if r < wr { wr = r; worst = Some(i); }
                }
                if let Some(w) = worst {
                    if wr < 0.98 && self.heroes[hi].cd <= 0.0 {
                        let support = *self.synergies.get(&Tag::Support).unwrap_or(&false);
                        let amt = (20.0 + self.heroes[hi].level as f32 * 6.0) * if support { 1.5 } else { 1.0 };
                        self.heal_hero(w, amt);
                        let (hx, hy) = (self.heroes[hi].x, self.heroes[hi].y);
                        let (wx, wy) = (self.heroes[w].x, self.heroes[w].y);
                        let color = self.hero_defs[self.heroes[hi].def_idx].color;
                        self.add_beam(hx, hy, wx, wy, color, 0.2);
                        let aspd = self.heroes[hi].aspd;
                        self.heroes[hi].cd = 1.0 / aspd;
                        self.heroes[hi].ult_charge += amt * 0.2;
                    }
                }
            } else if let Some(ti) = self.pick_target(hi) {
                if self.heroes[hi].cd <= 0.0 {
                    self.hero_attack(hi, ti);
                    let aspd = self.heroes[hi].aspd;
                    self.heroes[hi].cd = 1.0 / aspd;
                }
            }
        }

        // снаряды героев
        for pi in 0..self.projectiles.len() {
            let p = self.projectiles[pi].clone();
            if p.target_idx >= self.enemies.len() || self.enemies[p.target_idx].hp <= 0.0 { continue; }
            let (tx, ty) = (self.enemies[p.target_idx].x, self.enemies[p.target_idx].y);
            let d = ((tx - p.x).powi(2) + (ty - p.y).powi(2)).sqrt();
            if d < 14.0 {
                self.damage_enemy(p.target_idx, p.dmg, 0.0);
                self.enemies[p.target_idx].slow = self.enemies[p.target_idx].slow.max(1.2);
                self.add_particles(tx, ty, p.color, 5);
                self.projectiles[pi].speed = -1.0;
            } else {
                let step = p.speed * dt;
                self.projectiles[pi].x += (tx - p.x) / d * step;
                self.projectiles[pi].y += (ty - p.y) / d * step;
            }
        }
        self.projectiles.retain(|p| p.speed > 0.0);

        // враги
        let mut new_enemies: Vec<Enemy> = Vec::new();
        let mut hero_damage_queue: Vec<(usize, f32)> = Vec::new();
        for ei in 0..self.enemies.len() {
            if self.enemies[ei].hp <= 0.0 { continue; }
            self.enemies[ei].flash = (self.enemies[ei].flash - dt).max(0.0);

            if self.enemies[ei].burn > 0.0 {
                self.enemies[ei].burn -= dt;
                let dmg = self.enemies[ei].max_hp * 0.03 * dt;
                self.enemies[ei].hp -= dmg;
                if rand::gen_range(0.0_f32, 1.0) < 0.2 {
                    let (ex, ey) = (self.enemies[ei].x, self.enemies[ei].y);
                    self.add_particles(ex, ey, Color::new(1.0, 0.42, 0.21, 1.0), 1);
                }
                if self.enemies[ei].hp <= 0.0 { continue; }
            }
            if self.enemies[ei].slow > 0.0 { self.enemies[ei].slow -= dt; }

            let kind = self.enemies[ei].kind;
            let (ex, ey) = (self.enemies[ei].x, self.enemies[ei].y);
            let speed_mult = if self.enemies[ei].slow > 0.0 { 0.5 } else { 1.0 };
            let spd = self.enemies[ei].speed * speed_mult;

            let nearest = self.nearest_hero(ex, ey);

            match kind {
                EnemyKind::Healer => {
                    self.enemies[ei].cd -= dt;
                    if self.enemies[ei].cd <= 0.0 {
                        let mut worst: Option<usize> = None;
                        let mut wr = 1.0;
                        for (i, o) in self.enemies.iter().enumerate() {
                            if i == ei || o.hp <= 0.0 { continue; }
                            let r = o.hp / o.max_hp;
                            if r < wr && ((o.x - ex).powi(2) + (o.y - ey).powi(2)).sqrt() < 220.0 { wr = r; worst = Some(i); }
                        }
                        if let Some(w) = worst {
                            self.enemies[w].hp = (self.enemies[w].hp + self.enemies[w].max_hp * 0.15).min(self.enemies[w].max_hp);
                            let (wx, wy) = (self.enemies[w].x, self.enemies[w].y);
                            self.add_beam(ex, ey, wx, wy, Color::new(0.4, 1.0, 0.6, 1.0), 0.3);
                        }
                        self.enemies[ei].cd = 1.0 / self.enemies[ei].aspd;
                    }
                    if let Some(h) = nearest {
                        let d = ((self.heroes[h].x - ex).powi(2) + (self.heroes[h].y - ey).powi(2)).sqrt();
                        if d > 230.0 {
                            self.enemies[ei].x += (self.heroes[h].x - ex) / d * spd * dt;
                            self.enemies[ei].y += (self.heroes[h].y - ey) / d * spd * dt;
                        } else if d < 150.0 {
                            self.enemies[ei].x += spd * 0.3 * dt;
                        }
                    }
                }
                EnemyKind::Bomber => {
                    if let Some(h) = nearest {
                        let d = ((self.heroes[h].x - ex).powi(2) + (self.heroes[h].y - ey).powi(2)).sqrt();
                        if d < 35.0 {
                            for hi in 0..self.heroes.len() {
                                if self.heroes[hi].hp > 0.0 {
                                    let dd = ((self.heroes[hi].x - ex).powi(2) + (self.heroes[hi].y - ey).powi(2)).sqrt();
                                    if dd < 120.0 { hero_damage_queue.push((hi, self.enemies[ei].atk + 25.0)); }
                                }
                            }
                            self.add_shockwave(ex, ey, 120.0, Color::new(1.0, 0.67, 0.0, 1.0), 0.5);
                            self.add_particles(ex, ey, Color::new(1.0, 0.67, 0.0, 1.0), 20);
                            self.add_shake(0.3, 6.0);
                            self.enemies[ei].hp = 0.0;
                        } else {
                            self.enemies[ei].x += (self.heroes[h].x - ex) / d * spd * dt;
                            self.enemies[ei].y += (self.heroes[h].y - ey) / d * spd * dt;
                        }
                    }
                }
                EnemyKind::Boss => {
                    if let Some(h) = nearest {
                        // фаза 2
                        if self.enemies[ei].boss_phase == 1 && self.enemies[ei].hp < self.enemies[ei].max_hp * 0.5 {
                            self.enemies[ei].boss_phase = 2;
                            self.enemies[ei].color = Color::new(1.0, 0.27, 0.53, 1.0);
                            self.enemies[ei].aspd *= 1.4;
                            self.enemies[ei].atk *= 1.2;
                            self.add_shake(0.5, 10.0);
                            for _ in 0..3 {
                                let ne = self.make_enemy(EnemyKind::Fast, self.wave, false);
                                new_enemies.push(ne);
                            }
                        }
                        let d = ((self.heroes[h].x - ex).powi(2) + (self.heroes[h].y - ey).powi(2)).sqrt();
                        if d > self.enemies[ei].range {
                            self.enemies[ei].x += (self.heroes[h].x - ex) / d * spd * dt;
                            self.enemies[ei].y += (self.heroes[h].y - ey) / d * spd * dt;
                        } else {
                            self.enemies[ei].cd -= dt;
                            if self.enemies[ei].cd <= 0.0 {
                                self.enemies[ei].cd = 1.0 / self.enemies[ei].aspd;
                                hero_damage_queue.push((h, self.enemies[ei].atk));
                            }
                        }
                        self.enemies[ei].telegraph_cd -= dt;
                        if self.enemies[ei].telegraph_cd <= 0.0 {
                            self.enemies[ei].telegraph_cd = if self.enemies[ei].boss_phase == 1 { 5.0 } else { 3.0 };
                            let targets: Vec<usize> = (0..self.heroes.len()).filter(|&i| self.heroes[i].hp > 0.0).collect();
                            if !targets.is_empty() {
                                let ti = targets[rand::gen_range(0, targets.len())];
                                self.telegraphs.push(Telegraph {
                                    x: self.heroes[ti].x, y: self.heroes[ti].y, r: 100.0,
                                    timer: 1.5, max_timer: 1.5, dmg: self.enemies[ei].atk * 2.5,
                                });
                            }
                        }
                        if self.enemies[ei].boss_phase == 2 {
                            self.enemies[ei].spawn_cd -= dt;
                            if self.enemies[ei].spawn_cd <= 0.0 {
                                self.enemies[ei].spawn_cd = 8.0;
                                for _ in 0..2 {
                                    let ne = self.make_enemy(EnemyKind::Fast, self.wave, false);
                                    new_enemies.push(ne);
                                }
                            }
                        }
                    }
                }
                EnemyKind::Archer => {
                    if let Some(h) = nearest {
                        let d = ((self.heroes[h].x - ex).powi(2) + (self.heroes[h].y - ey).powi(2)).sqrt();
                        if d > self.enemies[ei].range {
                            self.enemies[ei].x += (self.heroes[h].x - ex) / d * spd * dt;
                            self.enemies[ei].y += (self.heroes[h].y - ey) / d * spd * dt;
                        } else {
                            self.enemies[ei].cd -= dt;
                            if self.enemies[ei].cd <= 0.0 {
                                self.enemies[ei].cd = 1.0 / self.enemies[ei].asp