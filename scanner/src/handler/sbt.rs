/*
 SBT handler's main function is hanlder event and change logic status or notify upstream
function:
1. SBT lifetime, mint or burn
2. record the history for all event
3. maintain the status of lifetime.

minting --> minted --> burning --> burned
^    ｜                     |
|    ｜                     |         
-- mint_fail           burn_fail
*/


pub struct SBT {

}