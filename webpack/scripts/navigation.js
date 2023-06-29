const navbar = document.querySelector(".navigation-container");
const navDropDownButton = document.querySelector("#dropdown-navigation-button");
const navbarLinks = document.querySelector("#navbar-links");

window.addEventListener("scroll", () => {
  if (window.scrollY >= 60) {
    navbar.classList.add("scrolled");
    // if (navbarLinks.classList.contains("expanded")) {
    // }
    navbarLinks.classList.add("scrolled");
  } else {
    navbar.classList.remove("scrolled");
    navbarLinks.classList.remove("scrolled");
  }
});

window.addEventListener("resize", () => {
  if (window.innerWidth >= 530) {
    navbarLinks.classList.remove("expanded");
  }
});

navDropDownButton.addEventListener("click", (e) => {
  navbarLinks.classList.toggle("expanded");
  e.stopPropagation();
});

document.addEventListener("click", (e) => {
  if (e.target.closest("#navbar")) return;
  navbarLinks.classList.remove("expanded");
});

document.addEventListener("touchmove", (e) => {
  if (e.target.closest("#navbar")) return;
  navbarLinks.classList.remove("expanded");
});