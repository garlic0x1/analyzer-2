<?php

function test($parameter) {
	query(intval(htmlspecialchars($_GET[0])));
	query(intval(htmlspecialchars($parameter)));
	$local_var = $_GET;
}

?>
