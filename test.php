<?php
$input = $_GET[0];
$value = $_GET;
if (false) {
	$value = intval($input);
}
query($value);
$filtered = "abcdefg" . unknown(htmlspecialchars($input));

function test($p1, $p2) {
	
}

function test2($p1, $p2) {
	
}
query(test($input, $a));
$newval = test($input, $filtered);
?>
