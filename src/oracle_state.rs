/// This files relates to the state of the oracle/oracle pool.
use crate::node_interface::{get_scan_boxes, send_transaction};
use crate::oracle_config::{get_config_yaml};
use crate::{NanoErg, BlockHeight, EpochID};
use crate::scans::{save_scan_ids_locally, register_epoch_preparation_scan, register_live_epoch_scan, register_datapoint_scan, register_pool_deposit_scan};
use crate::encoding::{deserialize_string, deserialize_integer};
use std::path::Path;
use sigma_tree::chain::{ErgoBox, ErgoBoxCandidate};
use sigma_tree::ast::{CollPrim, Constant, ConstantVal};
use yaml_rust::{YamlLoader};

#[derive(Debug, Clone)]
/// Enum for the state that the oracle pool box is currently in
pub enum PoolBoxState { 
    Preparation,
    Epoch
}


/// A `Stage` is defined here by it's contract address & it's scan_id
#[derive(Debug, Clone)]
pub struct Stage {
    pub contract_address: String,
    pub scan_id: String,
}

/// Overarching struct which allows for acquiring the state of the whole oracle pool protocol
#[derive(Debug, Clone)]
pub struct OraclePool {
    /// Address of the local oracle running the oracle core
    pub local_oracle_address: String,
    /// Token IDs
    pub oracle_pool_nft: String,
    pub oracle_pool_participant_token: String,
    /// Stages
    pub epoch_preparation_stage: Stage,
    pub live_epoch_stage: Stage,
    pub datapoint_stage: Stage,
    pub pool_deposit_stage: Stage,
}

/// The state of the oracle pool when it is in the Oracle Pool Epoch stage
#[derive(Debug, Clone)]
pub struct EpochState {
    pub funds: NanoErg,
    pub epoch_id: EpochID,
    pub commit_datapoint_in_epoch: bool,
    pub epoch_ends: BlockHeight,
    pub latest_pool_datapoint: String,
}

/// The state of the oracle pool when it is in the Epoch Preparation stage
#[derive(Debug, Clone)]
pub struct PreparationState {
    pub funds: NanoErg,
    pub next_epoch_ends: BlockHeight,
    pub latest_pool_datapoint: String,
}

/// The state of the local oracle's Datapoint box
#[derive(Debug, Clone)]
pub struct DatapointState {
    datapoint: u64,
    from_epoch: EpochID,
}

/// The current UTXO-set state of all of the Pool Deposit boxes
#[derive(Debug, Clone)]
pub struct PoolDepositsState {
    number_of_boxes: u64,
    total_ergs: u64
}

impl OraclePool {

    /// Create a new `OraclePool` struct
    pub fn new() -> OraclePool {
        let config = &YamlLoader::load_from_str(&get_config_yaml()).unwrap()[0];

        let local_oracle_address = config["oracle_address"].as_str().expect("No oracle_pool_nft specified in config file.").to_string();
        let oracle_pool_nft = config["oracle_pool_nft"].as_str().expect("No oracle_pool_nft specified in config file.").to_string();
        let oracle_pool_participant_token = config["oracle_pool_participant_token"].as_str().expect("No oracle_pool_participant_token specified in config file.").to_string();
        
        let epoch_preparation_contract_address = config["epoch_preparation_contract_address"].as_str().expect("No epoch_preparation_contract_address specified in config file.").to_string();
        let live_epoch_contract_address = config["live_epoch_contract_address"].as_str().expect("No live_epoch_contract_address specified in config file.").to_string();
        let datapoint_contract_address = config["datapoint_contract_address"].as_str().expect("No datapoint_contract_address specified in config file.").to_string();
        let pool_deposit_contract_address = config["pool_deposit_contract_address"].as_str().expect("No pool_deposit_contract_address specified in config file.").to_string();

        // If scanIDs.json exists, skip registering scans & saving generated ids
        if !Path::new("scanIDs.json").exists() {
            let id1 = register_epoch_preparation_scan(&oracle_pool_nft, &epoch_preparation_contract_address);
            let id2 = register_live_epoch_scan(&oracle_pool_nft, &live_epoch_contract_address);
            let id3 = register_datapoint_scan(&oracle_pool_participant_token, &datapoint_contract_address, &local_oracle_address);
            let id4 = register_pool_deposit_scan(&pool_deposit_contract_address);

            save_scan_ids_locally(id1, id2, id3, id4);
        }

        // Read scanIDs.json for scan ids
        let scan_ids = json::parse(&std::fs::read_to_string("scanIDs.json").expect("Unable to read scanIDs.json")).expect("Failed to parse scanIDs.json");
        let epoch_preparation_scan_id = scan_ids["epoch_preparation_scan_id"].to_string();
        let live_epoch_scan_id = scan_ids["live_epoch_scan_id"].to_string();
        let datapoint_scan_id = scan_ids["datapoint_scan_id"].to_string();
        let pool_deposit_scan_id = scan_ids["pool_deposit_scan_id"].to_string();


        OraclePool {
            local_oracle_address: local_oracle_address,
            oracle_pool_nft: oracle_pool_nft,
            oracle_pool_participant_token: oracle_pool_participant_token,
            epoch_preparation_stage: Stage { contract_address: epoch_preparation_contract_address, scan_id: epoch_preparation_scan_id},
            live_epoch_stage: Stage { contract_address: live_epoch_contract_address, scan_id: live_epoch_scan_id },
            datapoint_stage: Stage { contract_address: datapoint_contract_address, scan_id: datapoint_scan_id },
            pool_deposit_stage: Stage { contract_address: pool_deposit_contract_address, scan_id: pool_deposit_scan_id },
        }


    }

    /// Get the current stage of the oracle pool box. Returns either `Preparation` or `Epoch`.
    pub fn check_oracle_pool_stage(&self) -> PoolBoxState {
        let epoch_preparation_box_list = get_scan_boxes(&self.epoch_preparation_stage.scan_id).unwrap_or(vec![]);

        if epoch_preparation_box_list.len() > 0 {
           return PoolBoxState::Preparation;
        }
        else {
           return PoolBoxState::Epoch;
        }
    }

    /// Get the state of the current oracle pool epoch
    pub fn get_live_epoch_state(&self) -> Option<EpochState> {
        let live_epoch_box_list = get_scan_boxes(&self.live_epoch_stage.scan_id)?;
        // let epoch_box = live_epoch_box_list.into_iter().nth(0)?;

        let datapoint_state = self.get_datapoint_state();
        // use datapoint_state.from_epoch() to get the oracle pool epoch box id to compare


        // The box id of the epoch that the oracle last posted a datapoint
        // let datapoint_r5 = datapoint_box.additional_registers.get_ordered_values()[1];

        // let epoch_box_id = ...

        // let commit_datapoint_in_epoch = box_id == datapoint_r5;

        // Latest pool datapoint is held in R4 of the epoch box
        // let latest_pool_datapoint = epoch_box.additional_registers.get_ordered_values()[0];

        // Block height epochs ends is held in R5 of the epoch box
        // let epoch_ends = epoch_box.additional_registers.get_ordered_values()[1];

        // let epoch_state = EpochState {
            // funds: epoch_box.value.0,
            // epoch_id: epoch_box_id,
            // commit_datapoint_in_epoch: commit_datapoint_in_epoch,
            // epoch_ends: epoch_ends
            // latest_pool_datapoint: latest_pool_datapoint,
        // }
        // Some(epoch_state)
        None
    }

    /// Get the state of the current epoch preparation box
    pub fn get_preparation_state(&self) -> Option<PreparationState> {
        let epoch_prep_box_list = get_scan_boxes(&self.epoch_preparation_stage.scan_id)?;
        // let epoch_prep_box = epoch_prep_box_list.into_iter().nth(0)?;

        // Latest pool datapoint is held in R4
        // let latest_pool_datapoint = epoch_prep_box.additional_registers.get_ordered_values()[0];

        // Next epoch ends height held in R5
        // let next_epoch_ends = epoch_prep_box.additional_registers.get_ordered_values()[1];


        // let prep_state = PreparationState {
        //     funds: epoch_prep_box.value.0,
        //     next_epoch_ends: next_epoch_ends,
        //     latest_pool_datapoint: latest_pool_datapoint,
        // }
        // Some(prep_state)


        None
    }

    /// Get the current state of the local oracle's datapoint
    pub fn get_datapoint_state(&self) -> Option<DatapointState> {
        let datapoint_box_list = get_scan_boxes(&self.datapoint_stage.scan_id)?;
        let datapoint_box = datapoint_box_list.into_iter().nth(0)?;
        let datapoint_box_regs = datapoint_box.additional_registers.get_ordered_values();

        // The Live Epoch box id of the epoch the datapoint was posted in (which is held in R5)
        let from_epoch = deserialize_string(&datapoint_box_regs[1])?;

        // Oracle datapoint held in R6
        let datapoint = deserialize_integer(&datapoint_box_regs[2])?;


        let datapoint_state = DatapointState {
            datapoint: datapoint as u64,
            from_epoch: from_epoch,
        };
        Some(datapoint_state)

    }

    /// Get the current state of all of the pool deposit boxes
    pub fn get_pool_deposits_state(&self) -> Option<PoolDepositsState> {
        let datapoint_box_list = get_scan_boxes(&self.pool_deposit_stage.scan_id)?;

        let sum_ergs = datapoint_box_list.iter().fold(0, |acc, b| acc + b.value.value());

        let deposits_state = PoolDepositsState {
            number_of_boxes: datapoint_box_list.len() as u64,
            total_ergs: sum_ergs,
        };

        println!("{:?}", deposits_state);
        Some(deposits_state)
    }




}