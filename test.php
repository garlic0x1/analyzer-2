<?php
$input = $_GET[0];
$value = "";
if (false) {
	$value = intval($input);
}
query($value);
$filtered = "abcdefg" . unknown(htmlspecialchars($input));


$newval = test($input, $filtered);
?>
