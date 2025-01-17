use super::*;

impl<T: Config> Pallet<T> {
    pub fn calculate_validator_score(
        subnet_id: u32,
        account_id: &T::AccountId,
        current_epoch: u32,
    ) -> u128 {
        let performance = ValidatorPerformanceData::<T>::get(subnet_id, account_id);
        
        // No score if never validated
        if performance.total_validations == 0 {
            return 0;
        }

        // Calculate base score from successful validation ratio
        let success_ratio = (performance.successful_validations as u128 * Self::PERCENTAGE_FACTOR) / 
            performance.total_validations as u128;

        // Penalize recent slashes more heavily
        let slash_penalty = if current_epoch.saturating_sub(performance.last_slash_epoch) < 100 {
            Self::PERCENTAGE_FACTOR / 2  // 50% penalty for recent slash
        } else {
            0
        };

        // Bonus for consecutive successful validations
        let consecutive_bonus = std::cmp::min(
            performance.consecutive_validations as u128 * (Self::PERCENTAGE_FACTOR / 100), // 1% per consecutive
            Self::PERCENTAGE_FACTOR / 4  // Max 25% bonus
        );

        // Final score calculation
        let final_score = success_ratio
            .saturating_sub(slash_penalty)
            .saturating_add(consecutive_bonus);

        std::cmp::min(final_score, Self::PERCENTAGE_FACTOR)
    }

    pub fn select_validator_weighted(
        subnet_id: u32,
        candidates: Vec<T::AccountId>,
        current_epoch: u32,
    ) -> Option<T::AccountId> {
        if candidates.is_empty() {
            return None;
        }

        // Calculate scores and total weight
        let mut total_weight: u128 = 0;
        let weighted_candidates: Vec<(T::AccountId, u128)> = candidates
            .into_iter()
            .map(|account_id| {
                let score = Self::calculate_validator_score(subnet_id, &account_id, current_epoch);
                total_weight = total_weight.saturating_add(score);
                (account_id, score)
            })
            .collect();

        if total_weight == 0 {
            // If all candidates have 0 score, fall back to random selection
            let rand_index = Self::get_random_number(weighted_candidates.len() as u32, current_epoch as u32);
            return Some(weighted_candidates[rand_index as usize].0.clone());
        }

        // Get random point in total weight
        let target_weight = Self::get_random_number(total_weight as u32, current_epoch as u32) as u128;
        
        // Find the selected validator
        let mut current_weight: u128 = 0;
        for (account_id, weight) in weighted_candidates {
            current_weight = current_weight.saturating_add(weight);
            if current_weight >= target_weight {
                return Some(account_id);
            }
        }

        // Fallback (shouldn't happen due to previous checks)
        Some(weighted_candidates[0].0.clone())
    }

    // Update validator performance after an epoch
    pub fn update_validator_performance(
        subnet_id: u32,
        validator: T::AccountId,
        success: bool,
        current_epoch: u32,
    ) {
        ValidatorPerformanceData::<T>::mutate(subnet_id, validator, |performance| {
            performance.total_validations = performance.total_validations.saturating_add(1);
            
            if success {
                performance.successful_validations = performance.successful_validations.saturating_add(1);
                performance.consecutive_validations = performance.consecutive_validations.saturating_add(1);
            } else {
                performance.consecutive_validations = 0;
            }
        });
    }

    // Record slash event
    pub fn record_validator_slash(
        subnet_id: u32,
        validator: T::AccountId,
        current_epoch: u32,
    ) {
        ValidatorPerformanceData::<T>::mutate(subnet_id, validator, |performance| {
            performance.last_slash_epoch = current_epoch;
            performance.total_slashes = performance.total_slashes.saturating_add(1);
            performance.consecutive_validations = 0;
        });
    }
}