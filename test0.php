<?php
$input = unknown($_GET[0]);
$value = $_GET;
if (false) {
	$value = intal($input);
}
query($value);
$value = $input . "aaa";
//flag_query($value);

$filtered = "abcdefg" . unknown(htmlspecialchars($input));

function test($p1, $p2) {
	$local_var = $p2;	
	test($p1, $p2);
	return ($p1);
}

function test1($p1, $p2) {
	$local_var = $p2;	
	//return ($p1);
	query($_GET[]);
}

add_action("test1");

$newval = test($value, $filtered);
$newval2 = test($value, $filtered);
query($newval);
$notaint = test();
?>
