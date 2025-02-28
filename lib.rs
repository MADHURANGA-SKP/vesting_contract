#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod vesting {

    #[ink(storage)]
    pub struct VestingContract {
        realeasable_balance: Balance,
        released_balance: Balance,
        duration_time: Timestamp,
        start_time: Timestamp,
        benificiary: AccountId,
        owner: AccountId,
    }

    //error when benificiary is zero address
    //and error when the realeasble balance is zero
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        InvalidBenificiary,
        ZeroReleasbleBalance,
    }

    //evenet for emit when a release is made
    #[ink(event)]
    pub struct Released{
        value: Balance,
        to: AccountId,
    }

    // # this is to set the follwing during contract development
    //-benificiary: the account that will recive the tokens
    //-duration_time: duration that will recive the tokens 
    //please not this is in seconds
    //-start_time: the time at which point vesting start
    //-owner: the account that can release the tokesns
    //-releasable_balance: the initial amount of tokems vested
    //--releasable_blance: the initial amount of token releasd
    //
    //the benificiary connot be the zero address
    impl VestingContract {
        #[ink(constructor, payable)]
        pub fn new(
            benificiary: AccountId,
            duration_time_in_sec: Timestamp,
        ) -> Result<Self, Error> {
            if benificiary == AccountId::from([0x0; 32]){
                return Err(Error::InvalidBenificiary)
            }

            //this is multiply by 1000 to confirm to the time stamp format
            let duration_time = duration_time_in_sec.checked_mul(1000).unwrap();

            let start_time = Self::env().block_timestamp();
            let owner = Self::env().caller();
            let releasable_balance = 0;
            let released_balance = 0;

            Ok(Self {
                duration_time,
                start_time,
                benificiary,
                releasable_balance,
                released_balance,
            })
        }

        //returns current time stamp
        pub fn time_now(&self) -> Timestamp {
            self.env().block_timestamp()
        }

        //this returns this contract balance
        #[ink(message)]
        pub fn this_contract_balance(&self) -> Balance {
            self.env().balance()
        }

        //returns benficiary wallet addr
        #[ink(message)]
        pub fn benificiary(&self) -> AccountId{
            self.benificiary
        }

        //returns the time at which poing vesting starts
        #[ink(message)]
        pub fn start_time(&self) -> Timestamp {
            self.start_time
        }

        //this returns the durtion of the vesting period in seconds
        #[ink(message)]
        pub fn duration_time(&self) -> Timestamp {
            self.duration_time
        }

        //returns the time at which point vesting ends
        #[ink(message)]
        pub fn end_time(&self) -> Timestamp {
            self.start_time().checked_add(self.duration_time()).unwarap()
        }

        //returns the amount of time that remain until the end of vesting period
        #[ink(message)]
        pub fn time_remaining(&self) -> Timestamp {
            if self.time_now() < self.end_time() {
                self.end_time().checked_sub(self.time_now()).unwrap()
            } else {
                0
            }
        }

        //returns the amount of native tokens that has already vested
        #[ink(message)]
        pub fn released_balance(&self) -> Balance {
            self.released_balance
        }

        //returns the amount of native tokens that is currently available for relese
        #[ink(message)]
        pub fn releasable_balance(&self) -> Balance {
            (self.vested_amount() as Balance)
            .checked_sub(self.released_balance())
            .unwrap()
        }

        //calculate the amount tha thas already vested
        //but hasn;t been releasd from the contract yet
        #[ink(message)]
        pub fn vested_amount(&self)  -> Balance {
            self.vesting_schedule(self.this_contract_balance(), self.time_now())
        }

        //sends the releaseble balance to the benificiary
        //wallet address, no matter who trigger the release
        #[ink(message)]
        pub fn release(&mut self) -> Result<(), Error> {
            let releaseble = self.realeasable_balance();
            if releaseble == 0 {
                return Err(Error::ZeroReleasbleBalance)
            }

            self.released_balance = 
                self.released_balance.checked_add(releaseble).unwrap();
            self.env()
                .transfer(self.benificiary, realeasble)
                .expect("Transfer faild during release")

            self.env().emit_event(Released {
                value; releaseble,
                to: self.benficiary,
            });

            Ok(())
        }

        //this calculate the amount of toekn that have vested up
        //to given current_time
        //
        //vesting shecdule is linear, meaning tokens are releasd evenly over
        //the vesting duration
        //
        //#parameters:
        //-total_alocation: the total number of tokens allocated for vesting
        //-current_time: the current timestamp for which we want to check the vested ammount
        //
        //#returns
        //-'0' if the current_time is before the vesting start time
        // total_allocation if the current_time is after the vesting
        //end time or at least equal to it
        //-A prorated amount based on how much time has passed since
        //the start of the vesting period o=if the current_time is
        //during the versting period
        //
        //# Example:
        //If the vesting duration is 200 seconds and 100 seconds have
        //passed since the start time, then 50% of the total_allocation
        //would have vested.
        pb fn vesting_schedule(
            &self,
            total_allocation: Balance,
            current_time: Timestamp,
        ) -> Balance {
            if current_time < self.start_time() {
                0
            } else if current_time >= self.end_time() {
                return total_allocation
            } else {
                return (total_allocation.checked_mul(
                    (current_time.checked_sub(self.start_time()).unwrap()) as Balance,
                ))
                .unwrap()
                .checked_div(self.duration_time() as Balance)
                .unwrap()
            }
        }
    }
}
