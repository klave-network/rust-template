use alloy_sol_types::sol;

sol! {
    function mint(address to, uint256 amount) external;
    function burn(address to, uint256 amount) external;
    function pause() external;
    function unpause() external;
}        