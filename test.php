<?php
$input = $_GET[0];
query($input);
$filtered = htmlspecialchars($input);
function test_func() {
	$inner_scope = $_GET;
	echo $inner_scope;
}

$end = 0;
?>
