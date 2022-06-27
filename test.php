<?php
$input = $_GET[0];
if (true) {
	$value = $input;
}
if (false) {
	$value = intval($input);
}
$filtered = "abcdefg" . unknown(htmlspecialchars($input));

query(intval($filtered));
$dead = $filtered;

test($filtered);
test($input);
?>
