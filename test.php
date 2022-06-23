<?php
$input = htmlspecialchars($_GET[0]);
query($input);
function test_func() {
	$inner_scope = $_GET;
	echo $inner_scope;
}

$end = 0;
?>
