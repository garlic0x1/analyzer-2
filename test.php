<?php
$input = $_GET[0];
$value = "";
if (true) {
	$value = $input;
}
if (false) {
	$value = intval($input);
}
query($value);
$filtered = "abcdefg" . unknown(htmlspecialchars($input));

query(intval($filtered));
$dead = $filtered;

test($filtered);
test($input);
?>
