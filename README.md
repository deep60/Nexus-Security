# Nexus-Security
Central hub for threat intelligence and Blockchain-based threat intelligence
graph TD

    15["User<br>External Actor"]
    49["Database<br>External Service"]
    50["Redis Cache<br>External Service"]
    subgraph 1["Frontend System<br>React"]
        subgraph 2["Source Code<br>React/TypeScript"]
            45["App.tsx<br>React/TypeScript"]
            46["index.tsx<br>React/TypeScript"]
            47["App.css<br>CSS"]
            48["index.css<br>CSS"]
        end
        subgraph 3["Public Assets<br>Static"]
            42["index.html<br>HTML"]
            43["favicon.ico<br>ICO"]
            44["logo192.png<br>PNG"]
        end
    end
    subgraph 4["Blockchain System<br>Hardhat/Solidity"]
        38["Smart Contracts<br>Solidity"]
        39["Deployment Modules<br>TypeScript"]
        40["Tests<br>TypeScript"]
        41["Hardhat Config<br>TypeScript"]
    end
    subgraph 5["Backend System<br>Rust"]
        37["Shared Library<br>Rust"]
        subgraph 10["API Gateway<br>Rust"]
            subgraph 11["Utilities<br>Rust"]
                25["Crypto<br>Rust"]
                26["Validation<br>Rust"]
            end
            subgraph 12["Services<br>Rust"]
                22["Blockchain<br>Rust"]
                23["Database<br>Rust"]
                24["Redis<br>Rust"]
            end
            subgraph 13["Models<br>Rust"]
                19["Analysis<br>Rust"]
                20["Bounty<br>Rust"]
                21["User<br>Rust"]
            end
            subgraph 14["Handlers<br>Rust"]
                16["Auth<br>Rust"]
                17["Bounty<br>Rust"]
                18["Submission<br>Rust"]
            end
        end
        subgraph 6["Bounty Manager<br>Rust"]
            36["Reputation<br>Rust"]
            subgraph 7["Handlers<br>Rust"]
                32["Bounty<br>Rust"]
                33["Payout<br>Rust"]
                34["Reputation<br>Rust"]
                35["Submission<br>Rust"]
            end
        end
        subgraph 8["Analysis Engine<br>Rust"]
            30["Analysis Result<br>Rust"]
            31["File Handler<br>Rust"]
            subgraph 9["Analyzers<br>Rust"]
                27["Hash<br>Rust"]
                28["Static<br>Rust"]
                29["YARA<br>Rust"]
            end
        end
        %% Edges at this level (grouped by source)
        10["API Gateway<br>Rust"] -->|Manages| 6["Bounty Manager<br>Rust"]
        10["API Gateway<br>Rust"] -->|Requests analysis| 8["Analysis Engine<br>Rust"]
        10["API Gateway<br>Rust"] -->|Uses| 37["Shared Library<br>Rust"]
        8["Analysis Engine<br>Rust"] -->|Provides results to| 10["API Gateway<br>Rust"]
        8["Analysis Engine<br>Rust"] -->|Uses| 37["Shared Library<br>Rust"]
        6["Bounty Manager<br>Rust"] -->|Uses| 37["Shared Library<br>Rust"]
    end
    %% Edges at this level (grouped by source)
    15["User<br>External Actor"] -->|Interacts with| 1["Frontend System<br>React"]
    10["API Gateway<br>Rust"] -->|Interacts with| 4["Blockchain System<br>Hardhat/Solidity"]
    10["API Gateway<br>Rust"] -->|Reads/Writes| 49["Database<br>External Service"]
    10["API Gateway<br>Rust"] -->|Caches with| 50["Redis Cache<br>External Service"]
    1["Frontend System<br>React"] -->|Uses API| 10["API Gateway<br>Rust"]
    6["Bounty Manager<br>Rust"] -->|Reads/Writes| 49["Database<br>External Service"]
