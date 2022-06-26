<?php
$input = $_GET[0];
if (true) {
query(intval($input));
}
$filtered = "abcdefg" . unknown(htmlspecialchars($input));

function test($parameter) {
	query(intval(htmlspecialchars($_GET[0])));
	query(intval(htmlspecialchars($parameter)));
}

query(intval($filtered));
$dead = $filtered;

test();
?>
