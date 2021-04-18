import {Directive, ElementRef, HostListener, OnDestroy, OnInit} from '@angular/core';
import {NgControl} from '@angular/forms';
import {Subscription} from 'rxjs';

@Directive({
  selector: '[appFormValidity]'
})
export class FormValidityDirective implements OnInit, OnDestroy {

  subs = new Subscription();

  constructor(private el: ElementRef, private control: NgControl) {
  }

  ngOnInit(): void {
    this.subs.add(this.control.valueChanges.subscribe(_ => this.checkValidity()));
  }

  ngOnDestroy(): void {
    this.subs.unsubscribe();
  }

  // Check for autocompletion
  @HostListener('change')
  change(): void {
    this.checkValidity();
  }

  checkValidity(): void {
    if (this.control.errors && this.control.dirty) {
      this.el.nativeElement.classList.add('is-invalid');
    } else {
      this.el.nativeElement.classList.remove('is-invalid');
    }
  }

}
