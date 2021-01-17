import {AfterViewInit, Component, ElementRef, OnInit, ViewChild} from '@angular/core';

@Component({
  selector: 'app-navigation',
  templateUrl: './navigation.component.html',
  styleUrls: ['./navigation.component.scss']
})
export class NavigationComponent implements OnInit {

  constructor() {
  }

  ngOnInit(): void {
  }

  toggleNavbar(navbarId: string): void {
    const target = document.getElementById(navbarId);

    if (target) {
      target.classList.toggle('show');
    }
  }
}
