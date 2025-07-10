use noisy_float::prelude::*;
use rand::prelude::*;
use rand_distr::{Exp, Distribution};

use std::f64::INFINITY;
const EPSILON: f64 = 1e-8;

const DEBUG: bool = false;

#[derive(Copy, Clone, Debug)]
enum Dist {
    Hyperexp(f64, f64, f64),
    Uniform(f64, f64),
}

impl Dist {
    fn sample<R: Rng>(&self, rng: &mut R) -> f64 {
        match self {
            Dist::Hyperexp(low_mu, high_mu, prob_low) => {
                let mu = if rng.random::<f64>() < *prob_low {
                    low_mu
                } else {
                    high_mu
                };
                Exp::new(*mu).unwrap().sample(rng)
            }
            Dist::Uniform(low, high) => {
                rng.random_range(*low..*high)
            }
        }
    }
    fn mean(&self) -> f64 {
        use Dist::*;
        match self {
            Hyperexp(low_mu, high_mu, prob_low) => prob_low / low_mu + (1.0 - prob_low) / high_mu,
            Uniform(low, high) => (low + high) / 2.0
        }
    }
}

#[derive(Debug)]
struct Job {
    arrival_time: f64,
    rem_size: f64,
    original_size: f64,
}

#[derive(Debug)]
enum Policy {
    SRPT,
    PSJF,
    RS,
    // Two jobs below, one above second
    SRPTExcept(f64),
}

impl Policy {
    // presort by remaining size
    fn jobs_duration(&self, queue: &[Job], num_servers: usize) -> (Vec<usize>, f64) {
        assert_eq!(num_servers, 2);
        let (jobs, service_duration) = match self {
            Policy::SRPT | Policy::PSJF | Policy::RS => {
                let jobs: Vec<usize> = (0..queue.len()).take(num_servers).collect();
                let duration = jobs
                    .iter()
                    .map(|&j| queue[j].rem_size)
                    .min_by_key(|&r| n64(r))
                    .unwrap_or(INFINITY);
                (jobs, duration)
            }
            Policy::SRPTExcept(cut) => {
                if queue.len() != 3 {
                    let jobs = (0..queue.len()).take(num_servers).collect();
                    let duration = queue.first().map_or(INFINITY, |j| j.rem_size);
                    (jobs, duration)
                } else {
                    let r1 = queue[0].rem_size;
                    let r2 = queue[1].rem_size;
                    let r3 = queue[2].rem_size;
                    if r2 <= cut + EPSILON && r3 >= cut - EPSILON {
                        // Fix: Don't allow r3 to age down to below cut
                        (vec![0, 2], r1.min(r3 - cut + 2.0 * EPSILON))
                    } else if r2 - cut < r1 && r3 - cut < r1 && r3 >= cut + EPSILON {
                        let duration = (r2 - cut).max(r3 - cut);
                        (vec![0, 1], duration + EPSILON)
                    } else {
                        (vec![0, 1], r1)
                    }
                }
            }
        };
        (jobs, service_duration * num_servers as f64)
    }
    fn index(&self, job: &Job) -> f64 {
        match self {
            Policy::SRPT | Policy::SRPTExcept(_) => job.rem_size,
            Policy::PSJF => job.original_size,
            Policy::RS => job.original_size * job.rem_size,
        }
    }
}

fn simulate(
    policy: &Policy,
    num_servers: usize,
    num_jobs: u64,
    dist: Dist,
    rho: f64,
    seed: u64,
) -> f64 {
    assert!((dist.mean() - 1.0).abs() < EPSILON);
    let mut queue: Vec<Job> = vec![];
    let mut num_completions = 0;
    let mut total_response = 0.0;
    let mut time = 0.0;
    let mut rng = StdRng::seed_from_u64(seed);
    let arrival_dist = Exp::new(rho).unwrap();
    let mut next_arrival_time = arrival_dist.sample(&mut rng);
    let mut total_work = 0.0;
    let mut num_arrivals = 0;
    while num_completions < num_jobs {
        queue.sort_by_key(|job| n64(policy.index(job)));
        let (service_indices, service_duration) = policy.jobs_duration(&queue, num_servers);
        let next_duration = service_duration.min(next_arrival_time - time);
        let was_arrival = next_duration < service_duration;
        time += next_duration;
        let mut removal_indices = vec![];
        for service_index in service_indices {
            let job = &mut queue[service_index];
            job.rem_size -= next_duration / num_servers as f64;
            if job.rem_size < EPSILON {
                removal_indices.push(service_index)
            }
        }
        for removal_index in removal_indices.into_iter().rev() {
            let job = queue.remove(removal_index);
            total_response += time - job.arrival_time;
            num_completions += 1;
        }
        if was_arrival {
            if DEBUG {
                total_work += queue.iter().map(|job| job.rem_size).sum::<f64>();
            }
            num_arrivals += 1;
            let new_size = dist.sample(&mut rng);
            let new_job = Job {
                rem_size: new_size,
                original_size: new_size,
                arrival_time: time,
            };
            queue.push(new_job);
            // Because sampling is performed only on arrivals,
            // the arrival process is not policy-dependent,
            // making comparison between two different policies more accurate.
            next_arrival_time = time + arrival_dist.sample(&mut rng);
        }
    }
    if DEBUG {
        println!("rho {} work {}", rho, total_work / num_arrivals as f64);
    }
    total_response / num_completions as f64
}

fn main() {
    let dists = vec![Dist::Hyperexp(1.9, 0.1, 0.95), Dist::Uniform(0.0, 2.0)];
    for dist in dists {
        //let dist = Dist::Hyperexp(1.0, 1.0, 1.0);
        let rhos = vec![
            0.01, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45, 0.5, 0.55, 0.6, 0.65, 0.7,
            0.72, 0.74, 0.76, 0.78, 0.8, 0.82, 0.84, 0.86, 0.88, 0.9, 0.903, 0.906, 0.91, 0.913,
            0.916, 0.92, 0.923, 0.926, 0.93, 0.933, 0.936, 0.94, 0.943, 0.946, 0.95, 0.952, 0.954,
            0.956, 0.958, 0.96, 0.962, 0.964, 0.966, 0.968, 0.97, 0.972, 0.974, 0.976, 0.978, 0.98,
            0.983, 0.986, 0.99, 0.993, 0.996,
        ];
        //let rhos = vec![0.1, 0.5, 0.8];
        let seed = 0;
        let num_jobs = 10_000_000;
        let num_servers = 2;
        println!(
            "num_jobs {} num_servers {} seed {} dist {:?}",
            num_jobs, num_servers, seed, dist
        );
        let policies = vec![
            Policy::SRPT,
            Policy::SRPTExcept(0.1),
            Policy::SRPTExcept(0.2),
            Policy::SRPTExcept(0.5),
            Policy::SRPTExcept(1.0),
            Policy::SRPTExcept(1.5),
            Policy::PSJF,
            Policy::RS,
        ];
        print!("rho");
        for policy in &policies {
            print!(";{:?}", policy);
        }
        println!();
        for rho in rhos {
            print!("{}", rho);
            for policy in &policies {
                let response = simulate(policy, num_servers, num_jobs, dist, rho, seed);
                print!(";{}", response);
            }
            println!();
        }
    }
}
