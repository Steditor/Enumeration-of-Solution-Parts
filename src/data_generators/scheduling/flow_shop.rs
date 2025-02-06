use crate::{
    data_structures::{
        scheduling_problems::{FlowShop, Job, SchedulingInstance},
        Index,
    },
    experiments::InstanceGenerator,
};

use super::taillard_lcg::TaillardLCG;

/// A flow shop generated according to Taillard \[1, 2\].
///
/// Each processing time is chosen uniformly at random from the integer interval `1..=99`.
///
/// \[1\] E. Taillard, „Benchmarks for basic scheduling problems“, European Journal of Operational Research, Bd. 64, Nr. 2, S. 278–285, Jan. 1993, doi: [10.1016/0377-2217(93)90182-M](https://doi.org/10.1016/0377-2217(93)90182-M).<br>
/// \[2\] E. Taillard, “Scheduling instances,” Éric Taillard’s page. \[Online\]. Available: <http://mistic.heig-vd.ch/taillard/problemes.dir/ordonnancement.dir/ordonnancement.html>.
pub struct Taillard {
    pub jobs: u32,
    pub machines: u32,
}

impl InstanceGenerator<SchedulingInstance<FlowShop, i32>> for Taillard {
    fn path() -> String {
        String::from("./data/scheduling/flowshop/taillard/")
    }

    fn file_name(&self) -> String {
        format!("{}_{}", self.jobs, self.machines)
    }

    fn generate(&self, seed: u64) -> SchedulingInstance<FlowShop, i32> {
        let mut rng = TaillardLCG::from_seed(seed);

        let mut job_data: Vec<Job<i32>> = (0..self.jobs)
            .map(|id| Job::for_num_operations(id, self.machines))
            .collect();

        for i in 0..self.machines {
            for j in &mut job_data {
                j.operations[i.index()] = rng.next_i32(1..=99);
            }
        }

        SchedulingInstance {
            environment: FlowShop {
                machines: self.machines,
            },
            jobs: job_data,
            precedences: (),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const TA001: [[i32; 20]; 5] = [
        [
            54, 83, 15, 71, 77, 36, 53, 38, 27, 87, 76, 91, 14, 29, 12, 77, 32, 87, 68, 94,
        ],
        [
            79, 3, 11, 99, 56, 70, 99, 60, 5, 56, 3, 61, 73, 75, 47, 14, 21, 86, 5, 77,
        ],
        [
            16, 89, 49, 15, 89, 45, 60, 23, 57, 64, 7, 1, 63, 41, 63, 47, 26, 75, 77, 40,
        ],
        [
            66, 58, 31, 68, 78, 91, 13, 59, 49, 85, 85, 9, 39, 41, 56, 40, 54, 77, 51, 31,
        ],
        [
            58, 56, 20, 85, 53, 35, 53, 41, 69, 13, 86, 72, 8, 49, 47, 87, 58, 18, 68, 28,
        ],
    ];

    #[test]
    fn test_ta001() {
        let seed = 873654221;
        let n = 20;
        let m = 5;

        let instance = Taillard {
            jobs: n,
            machines: m,
        }
        .generate(seed);

        for i in 0..m {
            for j in 0..n {
                assert_eq!(
                    instance.jobs[j.index()].operations[i.index()],
                    TA001[i.index()][j.index()]
                );
            }
        }
    }
}
