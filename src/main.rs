use noisy_float::prelude::*;
use rand::prelude::*;
use rand_distr::Exp;

use std::f64::INFINITY;
const EPSILON: f64 = 1e-8;

#[derive(Copy, Clone, Debug)]
enum Dist {
    Hyperexp(f64, f64, f64),
}

impl Dist {
    fn sample<R: Rng>(&self, rng: &mut R) -> f64 {
        match self {
            Dist::Hyperexp(low_mu, high_mu, prob_low) => {
                let mu = if rng.gen::<f64>() < *prob_low {
                    low_mu
                } else {
                    high_mu
                };
                Exp::new(*mu).unwrap().sample(rng)
            }
        }
    }
    fn mean(&self) -> f64 {
        use Dist::*;
        match self {
            Hyperexp(low_mu, high_mu, prob_low) => prob_low / low_mu + (1.0 - prob_low) / high_mu,
        }
    }
}

#[derive(Debug)]
struct Job {
    arrival_time: f64,
    rem_size: f64,
}

#[derive(Debug)]
enum Policy {
    SRPT,
    // Two jobs below first, one above second
    SRPTExcept(f64, f64),
}

impl Policy {
    // presort by remaining size
    fn jobs_duration(&self, queue: &[Job], num_servers: usize) -> (Vec<usize>, f64) {
        assert_eq!(num_servers, 2);
        let (jobs, service_duration) = match self {
            Policy::SRPT => {
                let jobs = (0..queue.len()).take(num_servers).collect();
                let duration = queue.first().map_or(INFINITY, |j| j.rem_size);
                (jobs, duration)
            }
            Policy::SRPTExcept(small_cut, big_cut) => {
                if queue.len() != 3 {
                    let jobs = (0..queue.len()).take(num_servers).collect();
                    let duration = queue.first().map_or(INFINITY, |j| j.rem_size);
                    (jobs, duration)
                } else {
                    let r1 = queue[0].rem_size;
                    let r2 = queue[1].rem_size;
                    let r3 = queue[2].rem_size;
                    if r2 <= small_cut + EPSILON && r3 >= big_cut - EPSILON {
                        // Fix: Don't allow r3 to age down to below big_cut
                        (vec![0, 2], r1.min(r3 - big_cut + 2.0 * EPSILON))
                    } else if r2 - small_cut < r1 && r3 - big_cut < r1 && r3 >= big_cut + EPSILON {
                        let duration = (r2 - small_cut).max(r3 - big_cut);
                        (vec![0, 1], duration + EPSILON)
                    } else {
                        (vec![0, 1], r1)
                    }
                }
            }
        };
        (jobs, service_duration * num_servers as f64)
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
    while num_completions < num_jobs {
        queue.sort_by_key(|job| n64(job.rem_size));
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
            let new_size = dist.sample(&mut rng);
            let new_job = Job {
                rem_size: new_size,
                arrival_time: time,
            };
            queue.push(new_job);
            // Because sampling is performed only on arrivals,
            // the arrival process is not policy-dependent,
            // making comparison between two different policies more accurate.
            next_arrival_time = time + arrival_dist.sample(&mut rng);
        }
    }
    total_response / num_completions as f64
}

fn main() {
    let dist = Dist::Hyperexp(1.9, 0.1, 0.95);
    //let dist = Dist::Hyperexp(1.0, 1.0, 1.0);
    let rhos = vec![
        0.01, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45, 0.5, 0.55, 0.6, 0.65, 0.7, 0.72,
        0.74, 0.76, 0.78, 0.8, 0.82, 0.84, 0.86, 0.88, 0.9, 0.903, 0.906, 0.91, 0.913, 0.916, 0.92,
        0.923, 0.926, 0.93, 0.933, 0.936, 0.94, 0.943, 0.946, 0.95, 0.952, 0.954, 0.956, 0.958,
        0.96, 0.962, 0.964, 0.966, 0.968, 0.97, 0.972, 0.974, 0.976, 0.978, 0.98, 0.983, 0.986,
        0.99, 0.993, 0.996,
    ];
    let seed = 0;
    let num_jobs = 100_000_000;
    let num_servers = 2;
    println!(
        "num_jobs {} num_servers {} seed {} dist {:?}",
        num_jobs, num_servers, seed, dist
    );
    let policies = vec![Policy::SRPT, Policy::SRPTExcept(4.0, 4.0)];
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
