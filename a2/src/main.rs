use clap::Parser;
use std::collections::HashMap;
use std::collections::HashSet;
use crate::LogFormat::Linux;
use crate::LogFormat::OpenStack;
use crate::LogFormat::Spark;
use crate::LogFormat::HDFS;
use crate::LogFormat::HPC;
use crate::LogFormat::Proxifier;
use crate::LogFormat::Android;
use crate::LogFormat::HealthApp;

mod packages;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
   /// Name of the raw logfile to convert to a CSV
   #[arg(long)]
   raw_linux: Option<String>,

   #[arg(long)]
   raw_openstack: Option<String>,

   #[arg(long)]
   raw_spark: Option<String>,

   #[arg(long)]
   raw_hdfs: Option<String>,

   #[arg(long)]
   raw_hpc: Option<String>,

   #[arg(long)]
   raw_proxifier: Option<String>,

   #[arg(long)]
   raw_android: Option<String>,

   #[arg(long)]
   raw_healthapp: Option<String>,

   #[arg(long)]
   to_parse: String,

   #[arg(long)]
   before: Option<String>,

   #[arg(long)]
   after: Option<String>,

   #[arg(long)]
   before_line: Option<String>,

   #[arg(long)]
   after_line: Option<String>,

   #[arg(long,default_value="3")]
   cutoff: Option<i32>,

   #[arg(long,require_equals=true,num_args=0..=1,default_missing_value_os="true")]
   single_map: Option<bool>,

   #[arg(long,default_value="8")]
   num_threads: Option<u32>,
}

#[test]
fn test_derive_2grams_from_trigram() {
    let twograms_oracle = vec![("one^two"), 
                               ("two^three")];
    let twograms = derive_2grams_from_trigram("one^two^three");
    assert_eq!(twograms_oracle, twograms);
}

fn derive_2grams_from_trigram(trigram:&str) -> Vec<String> {
    let grams : Vec<&str> = trigram.split("^").collect();
    return vec![format!("{}^{}", grams[0], grams[1]), 
                format!("{}^{}", grams[1], grams[2])];
}

pub enum LogFormat {
    Linux,
    OpenStack,
    Spark,
    HDFS,
    HPC,
    Proxifier,
    Android,
    HealthApp,
}

fn view_double_and_triple_dicts(double_dict:&HashMap<String, i32>, triple_dict:&HashMap<String, i32>) {
    packages::parser::print_dict("double", double_dict);
    packages::parser::print_dict("triple", triple_dict);
}

fn main() {
    let args = Args::parse();

    let mut input_fn = None;
    let mut log_format_opt = None;
    // hey, please let me know (email) if there's a more idiomatic way to do this
    if let Some(raw_linux) = args.raw_linux {
        log_format_opt = Some(Linux);
        input_fn = Some(raw_linux);
    } else if let Some(raw_openstack) = args.raw_openstack {
        log_format_opt = Some(OpenStack);
        input_fn = Some(raw_openstack);
    } else if let Some(raw_spark) = args.raw_spark {
        log_format_opt = Some(Spark);
        input_fn = Some(raw_spark);
    } else if let Some(raw_hdfs) = args.raw_hdfs {
        log_format_opt = Some(HDFS);
        input_fn = Some(raw_hdfs);
    } else if let Some(raw_hpc) = args.raw_hpc {
        log_format_opt = Some(HPC);
        input_fn = Some(raw_hpc);
    } else if let Some(raw_proxifier) = args.raw_proxifier {
        log_format_opt = Some(Proxifier);
        input_fn = Some(raw_proxifier);
    } else if let Some(raw_android) = args.raw_android {
        log_format_opt = Some(Android);
        input_fn = Some(raw_android);
    } else if let Some(raw_healthapp) = args.raw_healthapp {
        log_format_opt = Some(HealthApp);
        input_fn = Some(raw_healthapp);
    }
    let log_format = match log_format_opt {
        None => panic!("must specify a raw input file"),
        Some(lf) => lf,
    };
    let cutoff = args.cutoff.unwrap();

    let (double_dict, triple_dict, _all_token_list) =
        packages::parser::parse_raw(input_fn.unwrap(), &log_format);

    view_double_and_triple_dicts(&double_dict, &triple_dict);

    let (format_string_re, censored_regexps) =
        (packages::parser::regex_generator(packages::parser::format_string(&log_format)), packages::parser::censored_regexps(&log_format));

    //let sample_string = "Jun 23 23:30:05 combo sshd(pam_unix)[26190]: authentication failure; logname= uid=0 euid=0 tty=NODEVssh ruser= rhost=218.22.3.51  user=root authentication".to_string();
    // add befores and afters to the sample string, yielding extended_sample_string
    let mut sample_string_tokens = packages::parser::token_splitter(args.to_parse, 
                                                                    &format_string_re,
                                                                    &censored_regexps);
    let mut befores = match (args.before, args.before_line) {
        (None, None) => vec![],
        (Some(b), None) => b.split_whitespace().map(|s| s.to_string()).collect(),
        (None, Some(b)) | (Some(_), Some(b)) => {
            let r = packages::parser::token_splitter(b,
                                                     &format_string_re,
                                                     &censored_regexps);
            r[r.len()-2..r.len()].to_vec()
        }
    };
    let mut afters = match (args.after, args.after_line) {
        (None, None) => vec![],
        (Some(a), None) => a.split_whitespace().map(|s| s.to_string()).collect(),
        (None, Some(a)) | (Some(_), Some(a)) => {
            let r = packages::parser::token_splitter(a,
                                                     &format_string_re,
                                                     &censored_regexps);
            r[0..2].to_vec()
        }
    };

    let mut extended_sample_string_tokens = vec![];
    extended_sample_string_tokens.append(&mut befores);
    extended_sample_string_tokens.append(&mut sample_string_tokens);
    extended_sample_string_tokens.append(&mut afters);
    println!("{:?}", extended_sample_string_tokens);

    // collect 3-grams from extended_sample_string that occur less often than cutoff in the corpus
    let mut uncommon_3grams = vec![];

    for triple in extended_sample_string_tokens.windows(3) {
        let three_gram = format!("{}^{}^{}", triple[0], triple[1], triple[2]);
        if triple_dict.contains_key(&three_gram) && triple_dict.get(&three_gram).unwrap() < &cutoff {
            // println!("3-gram {}, count {}", three_gram, &triple_dict.get(&three_gram).unwrap());
            uncommon_3grams.push(three_gram);
        }
    }

    let mut deduped_2grams_from_uncommon_3grams : HashSet<String> = HashSet::new();
    for three_g in uncommon_3grams {
        for two_g in derive_2grams_from_trigram(&three_g) {
            deduped_2grams_from_uncommon_3grams.insert(two_g);
        }
    }
    let mut uncommon_2grams : Vec<String> = vec![];
    for two_g in deduped_2grams_from_uncommon_3grams {
        let two_g_count = double_dict.get(&two_g).unwrap();
        println!("2-gram {}, count {}", two_g, two_g_count);
        if two_g_count < &cutoff {
            uncommon_2grams.push(two_g);
            // println!("2-gram {}, count {}", two_g, two_g_count);
        }
    }

    // now, iterate on the original tokens again and look for uncommon 2grams that appear 
    let mut dynamic_tokens = vec![];
    for triple in extended_sample_string_tokens.windows(3) {
        let two_gram1 = format!("{}^{}", triple[0], triple[1]);
        let two_gram2 = format!("{}^{}", triple[1], triple[2]);
        if uncommon_2grams.contains(&two_gram1) && uncommon_2grams.contains(&two_gram2) {
            dynamic_tokens.push(triple[1].to_string());
        }
        // println!("focus is {}, have {} {}, contains is {}/{}", triple[1], two_gram1, two_gram2, uncommon_2grams.contains(&two_gram1), uncommon_2grams.contains(&two_gram2));
    }
    println!("dynamic tokens: {:?}", dynamic_tokens);
}
