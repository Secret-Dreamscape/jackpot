# Jackpot contract
The jackpot contract is responsible for managing the jackpot


## Instantiating the contract
To instantiate the contract you need to pass the following parameters to it: 

| Parameter | Type         | Description                                    |
|-----------|--------------|------------------------------------------------|
| jackpots  | Vec<Jackpot> | The list of jackpots with their initial values |
| nft_addr  | HumanAddr    | The address to the NFT contract                |
| nft_hash  | String       | The code hash for the NFT contract             |

## Handle Functions
| Function name    | Description                                                                                                             | Admin Only? |
|------------------|-------------------------------------------------------------------------------------------------------------------------|-------------|
| Fund             | Add money to the jackpots                                                                                               | No          |
| LiberateFunds    | Move the funds to another contract (useful when updating the contract)                                                  | No          |
| UpdateComplexity | Changes the complexity of words that can be chosen for one of the jackpots                                              | Yes         |
| NextWord         | Picks a new word for the specified jackpot (can only be called after 2 days since the first person claimed the jackpot) | No          |
| ShowMeTheMoney   | Claims a jackpot given                                                                                                  | No          |

## Queries
| Function name | Description                             |
|---------------|-----------------------------------------|
| GetJackpots   | Gets a list of jackpots and their stats |