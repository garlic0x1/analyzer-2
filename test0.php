<?php
$input = unknown($_GET[0]);
$value = $_GET;
if (false) {
	$value = intval($input);
}
query($value);
$value = $input;
flag_query($value);

$filtered = "abcdefg" . unknown(htmlspecialchars($input));

function test($p1, $p2) {
	$local_var = $p2;	
	return ($p1);
}

$newval = test($value, $filtered);
$newval2 = test($value, $filtered);
$notaint = test();
?>