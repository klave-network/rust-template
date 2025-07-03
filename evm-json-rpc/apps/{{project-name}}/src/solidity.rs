use alloy_sol_types::sol;

sol! {
    function balanceOf(address owner) view returns (uint256);
    function name() view returns (string);
    function symbol() view returns (string);
    function decimals() view returns (uint8);
    function totalSupply() view returns (uint256);
    function owner() view returns (address);

    function mint(address to, uint256 amount) external;
    function burn(address to, uint256 amount) external;
    function pause() external;
    function unpause() external;
}
