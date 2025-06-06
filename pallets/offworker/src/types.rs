// No changes needed
use super::*;
use crate::profitability::calculate_avg_delegate_divs;
use sp_runtime::RuntimeDebug;

#[derive(Clone, Debug, PartialEq, Encode, Decode)]
pub struct ConsensusSimulationResult<T: pallet_subspace::Config> {
    pub cumulative_copier_divs: I64F64,
    pub cumulative_avg_delegate_divs: I64F64,
    pub copier_margin: I64F64,
    pub max_encryption_period: u64,
    pub creation_block: u64,
    pub _phantom: PhantomData<T>,
}

impl<T: pallet_subspace::Config> Default for ConsensusSimulationResult<T> {
    fn default() -> Self {
        Self {
            cumulative_copier_divs: I64F64::from_num(0),
            cumulative_avg_delegate_divs: I64F64::from_num(0),
            copier_margin: I64F64::from_num(0),
            max_encryption_period: 0,
            creation_block: pallet_subspace::Pallet::<T>::get_current_block_number(),
            _phantom: PhantomData,
        }
    }
}

impl<T: pallet_subspace::Config + pallet_subnet_emission::Config> ConsensusSimulationResult<T> {
    pub fn update(
        &mut self,
        yuma_output: ConsensusOutput<T>,
        copier_uid: u16,
        delegation_fee: Percent,
    ) {
        let avg_delegate_divs =
            calculate_avg_delegate_divs::<T>(&yuma_output, copier_uid, delegation_fee)
                .unwrap_or_default();

        let copier_divs = yuma_output
            .dividends
            .get(copier_uid as usize)
            .map(|&div| I64F64::from_num(div))
            .unwrap_or_default();

        self.cumulative_copier_divs = self
            .cumulative_copier_divs
            .checked_add(copier_divs)
            .unwrap_or(self.cumulative_copier_divs);

        self.cumulative_avg_delegate_divs = self
            .cumulative_avg_delegate_divs
            .checked_add(avg_delegate_divs)
            .unwrap_or(self.cumulative_avg_delegate_divs);
    }
}

pub struct ShouldDecryptResult<T: pallet_subspace::Config> {
    pub should_decrypt: bool,
    pub simulation_result: ConsensusSimulationResult<T>,
    pub delta: I64F64,
}

impl<T: pallet_subspace::Config> Default for ShouldDecryptResult<T> {
    fn default() -> Self {
        Self {
            should_decrypt: false,
            simulation_result: ConsensusSimulationResult::default(),
            delta: I64F64::from_num(0),
        }
    }
}

pub struct SimulationYumaParams<T: Config> {
    pub uid: u16,
    pub params: ConsensusParams<T>,
    pub decrypted_weights_map: BTreeMap<u16, Vec<(u16, u16)>>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
pub struct DecryptedWeightsPayload<Public, BlockNumber> {
    pub subnet_id: u16,
    pub decrypted_weights: Vec<BlockWeights>,
    pub delta: I64F64,
    pub block_number: BlockNumber,
    pub public: Public,
    pub forced_send_by_rotation: bool,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, scale_info::TypeInfo)]
pub struct KeepAlivePayload<Public, BlockNumber> {
    pub public_key: (Vec<u8>, Vec<u8>),
    pub block_number: BlockNumber,
    pub public: Public,
}

impl<T: SigningTypes> SignedPayload<T> for DecryptedWeightsPayload<T::Public, BlockNumberFor<T>> {
    fn public(&self) -> T::Public {
        self.public.clone()
    }
}

impl<T: SigningTypes> SignedPayload<T> for KeepAlivePayload<T::Public, BlockNumberFor<T>> {
    fn public(&self) -> T::Public {
        self.public.clone()
    }
}
