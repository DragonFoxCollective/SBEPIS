use std::sync::Arc;

use hashbrown::{HashMap, HashSet};
use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub struct Jack {
	potential_targets: Vec<Arc<Target>>,
	personal_values: Vec<PersonalValue>,
	allegiences: HashMap<Arc<Faction>, i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionType {
	name: String,
	beneficial_to_target: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
	action_type: Arc<ActionType>,
	target: Option<Arc<Target>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
	name: String,
	allegiences: HashMap<Arc<Faction>, i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Faction {
	name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalValue {
	name: String,
	weights: Vec<ActionWeight>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionWeight {
	action_type: Arc<ActionType>,
	weight: i32,
}

impl Jack {
	pub fn next_action(&self) -> Action {
		let mut actions: Vec<Action> = vec![];
		actions.extend(
			self.personal_values
				.iter()
				.flat_map(|v| v.weights.iter().map(|w| w.action_type.clone()))
				.flat_map(|v| -> Box<dyn Iterator<Item = Action>> {
					if v.beneficial_to_target.is_some() {
						Box::new(self.potential_targets.iter().map(move |t| Action {
							action_type: v.clone(),
							target: Some(t.clone()),
						}))
					} else {
						Box::new(
							vec![Action {
								action_type: v.clone(),
								target: None,
							}]
							.into_iter(),
						)
					}
				}),
		);

		let weights = actions
			.into_iter()
			.map(|a| {
				let personal_value_weight: i32 = self
					.personal_values
					.iter()
					.flat_map(|v| v.weights.iter())
					.filter(|w| w.action_type == a.action_type)
					.map(|w| w.weight)
					.sum();

				let allegience_weight = if let Some(target) = &a.target {
					self.allegiences
						.keys()
						.cloned()
						.collect::<HashSet<Arc<Faction>>>()
						.intersection(
							&target
								.allegiences
								.keys()
								.cloned()
								.collect::<HashSet<Arc<Faction>>>(),
						)
						.map(|f| self.allegiences[f] * target.allegiences[f])
						.sum::<i32>() * if a.action_type.beneficial_to_target.unwrap() {
						1
					} else {
						-1
					}
				} else {
					0
				};

				let weight = personal_value_weight + allegience_weight;
				(a, weight)
			})
			.collect::<Vec<_>>();

		println!("Weights: {:#?}", weights);

		weights
			.into_iter()
			.max_by_key(|(_, weight)| *weight)
			.map(|(action, _)| action)
			.unwrap_or(Action {
				action_type: NOTHING.clone(),
				target: None,
			})
	}
}

lazy_static! {
	pub static ref NOTHING: Arc<ActionType> = Arc::new(ActionType {
		name: "Nothing".to_string(),
		beneficial_to_target: None,
	});
}

fn main() {
	println!("No main method, only tests here");
}

#[cfg(test)]
mod tests {
	use super::*;

	#[allow(dead_code)]
	struct TestInfo {
		stab: Arc<ActionType>,
		do_midnight_crew_things: Arc<ActionType>,
		midnight_crew: Arc<Faction>,
		derse: Arc<Faction>,
		jack_faction: Arc<Faction>,
		diamonds_droog_faction: Arc<Faction>,
		carapacian_35_faction: Arc<Faction>,
		diamonds_droog: Arc<Target>,
		carapacian_35: Arc<Target>,
		jack: Jack,
	}

	fn test_info() -> TestInfo {
		let stab = Arc::new(ActionType {
			name: "Stab".to_string(),
			beneficial_to_target: Some(false),
		});
		let do_midnight_crew_things = Arc::new(ActionType {
			name: "Do midnight crew things".to_string(),
			beneficial_to_target: Some(true),
		});

		let midnight_crew = Arc::new(Faction {
			name: "Midnight Crew".to_string(),
		});
		let derse = Arc::new(Faction {
			name: "Derse".to_string(),
		});

		let jack_faction = Arc::new(Faction {
			name: "Jack".to_string(),
		});
		let diamonds_droog_faction = Arc::new(Faction {
			name: "Carapacian".to_string(),
		});
		let carapacian_35_faction = Arc::new(Faction {
			name: "Carapacian".to_string(),
		});

		let diamonds_droog = Arc::new(Target {
			name: "Diamonds Droog".to_string(),
			allegiences: {
				let mut map = HashMap::new();
				map.insert(midnight_crew.clone(), 1);
				map.insert(derse.clone(), 1);
				map.insert(jack_faction.clone(), 1);
				map.insert(diamonds_droog_faction.clone(), 5);
				map
			},
		});
		let carapacian_35 = Arc::new(Target {
			name: "Carapacian #35".to_string(),
			allegiences: {
				let mut map = HashMap::new();
				map.insert(midnight_crew.clone(), -1);
				map.insert(derse.clone(), 1);
				map.insert(jack_faction.clone(), -1);
				map.insert(carapacian_35_faction.clone(), 5);
				map
			},
		});

		let jack = Jack {
			potential_targets: vec![diamonds_droog.clone(), carapacian_35.clone()],
			personal_values: vec![
				PersonalValue {
					name: "Stabbing".to_string(),
					weights: vec![ActionWeight {
						action_type: stab.clone(),
						weight: 1,
					}],
				},
				PersonalValue {
					name: "Midnight Crew Succeed".to_string(),
					weights: vec![ActionWeight {
						action_type: do_midnight_crew_things.clone(),
						weight: 1,
					}],
				},
			],
			allegiences: {
				let mut map = HashMap::new();
				map.insert(midnight_crew.clone(), 1);
				map.insert(derse.clone(), 1);
				map.insert(jack_faction.clone(), 5);
				map.insert(diamonds_droog_faction.clone(), 1);
				map.insert(carapacian_35_faction.clone(), -1);
				map
			},
		};

		TestInfo {
			stab,
			do_midnight_crew_things,
			midnight_crew,
			derse,
			jack_faction,
			diamonds_droog_faction,
			carapacian_35_faction,
			diamonds_droog,
			carapacian_35,
			jack,
		}
	}

	#[test]
	fn jack_stabs_carapacian_by_default() {
		let test_info = test_info();

		assert_eq!(
			test_info.jack.next_action(),
			Action {
				action_type: test_info.stab.clone(),
				target: Some(test_info.carapacian_35.clone()),
			}
		);
	}

	#[test]
	fn jack_wants_midnight_crew_to_succeed_when_theres_no_carapacians_to_stab() {
		let mut test_info = test_info();
		test_info.jack.potential_targets = vec![test_info.diamonds_droog.clone()];

		assert_eq!(
			test_info.jack.next_action(),
			Action {
				action_type: test_info.do_midnight_crew_things.clone(),
				target: Some(test_info.diamonds_droog.clone()),
			}
		);
	}

	#[test]
	fn jack_does_nothing_when_theres_no_one_to_stab() {
		let mut test_info = test_info();
		test_info.jack.potential_targets = vec![];

		assert_eq!(
			test_info.jack.next_action(),
			Action {
				action_type: NOTHING.clone(),
				target: None,
			}
		);
	}
}
